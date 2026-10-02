import { onBeforeUnmount, onMounted, shallowRef } from "vue";

import { isFileDropInside, listenForFileDrops } from "../../../../foundation/file-drop/file-drop";
import { i18n } from "../../../../foundation/i18n/i18n";
import { createAsyncDisposer } from "../../../../foundation/lifecycle/async-disposer";
import type { SpaceImportSourceAccess } from "../../assets/asset-operations";
import { addImportedSpaceAsset } from "../../assets/import";
import { importSpaceBatch, type SpaceImportBatchOutcome } from "../../assets/import-batch";
import { spaceImportContext } from "../../assets/import-context";
import { scheduleSpaceAssetProcessing } from "../../assets/processing-scheduler";
import type { CanvasFileImportRequest, CanvasFileImportReceipt } from "../../canvas/contracts/canvas-editor";
import { importDroppedFiles } from "../../canvas/drop";
import type { getCanvasEditorSession } from "../../canvas/editor-session";
import { useNodeOperationGate } from "../../canvas/node-operation-gate";
import { pasteSpaceClipboardAt } from "../../canvas/selection-actions";
import { noteFromPlainText } from "../../canvas/text-import";
import { pasteSpaceClipboard } from "../../clipboard/paste";
import type { useSpaceStore } from "../../session/space-store";
import { captureSpaceUrl } from "../../sources/capture-url";
import { classifySpaceUrl } from "../../sources/classify-url";
import { urlCaptureContext } from "../../sources/url-capture-context";
import type { SpaceUrlCaptureProjection } from "../../sources/url-capture-projection";

interface SpaceCanvasImportOptions {
  store: ReturnType<typeof useSpaceStore>;
  editorSession: ReturnType<typeof getCanvasEditorSession>;
  dropTarget(): HTMLElement | null;
  newNoteTitle: string;
  reportError(error: unknown): void;
}

export function useSpaceCanvasImports(options: SpaceCanvasImportOptions) {
  const operations = useNodeOperationGate();
  const fileDropListener = createAsyncDisposer();
  const fileHover = shallowRef(false);
  const fileHoverPosition = shallowRef<{ x: number; y: number }>();

  async function insertText(text: string, point: { x: number; y: number }) {
    if (!operations.allows("create")) return;
    const source = classifySpaceUrl(text.trim());
    if (source) return insertUrl(source.url, point);
    await options.store.addContent(noteFromPlainText(text, options.newNoteTitle), point);
  }

  async function insertUrl(url: string, point: { x: number; y: number }) {
    if (!operations.allows("create")) return;
    void captureSpaceUrl(urlCaptureContext(options.store), url, point).catch(options.reportError);
  }

  function retryCapture(capture: SpaceUrlCaptureProjection) {
    if (!operations.allows("create")) return;
    options.store.finishUrlCapture(capture.id);
    void insertUrl(capture.url, capture.position);
  }

  async function pasteClipboard(point: { x: number; y: number }) {
    if (!operations.allows("create")) return false;
    const spaceId = options.store.activeSpaceId;
    if (!spaceId) throw new Error("A Space must be active to paste canvas content");
    return pasteSpaceClipboard({
      imageName: i18n.global.t("spaces.nodeTypes.image"),
      pasteNodes: (html) => pasteSpaceClipboardAt(options.store, point, html),
      importFiles: async (paths) => {
        await importBatch(
          paths.map((path, key) => ({ key, path, sourceAccess: "clipboard" as const })),
          point,
        );
      },
      insertText: (text) => insertText(text, point),
      addImage: async (image) => {
        const nodes = await addImportedSpaceAsset(image, point, (data, position) => {
          if (options.store.activeSpaceId !== spaceId) throw new Error("Canvas paste target changed");
          return options.store.addContent(data, position);
        });
        for (const node of nodes) {
          scheduleSpaceAssetProcessing({ context: options.store, spaceId, nodeId: node.id, data: node.data });
        }
      },
    });
  }

  async function importFiles(files: readonly CanvasFileImportRequest[]) {
    if (!operations.allows("create")) return [];
    const outcomes = await importBatch(
      files.map((file) => ({
        key: file.sourceId,
        path: file.path,
        position: file.position,
        sourceAccess: "scoped" as const,
      })),
      files[0]?.position ?? { x: 0, y: 0 },
    );
    return outcomes.filter(isImportedOutcome).map(({ key, nodes: [node] }) => ({
      sourceId: key,
      node,
    })) satisfies CanvasFileImportReceipt[];
  }

  async function importBatch<Key>(
    requests: readonly {
      key: Key;
      path: string;
      sourceAccess: SpaceImportSourceAccess;
      position?: { x: number; y: number };
    }[],
    origin: { x: number; y: number },
  ) {
    const spaceId = options.store.activeSpaceId;
    if (!spaceId) throw new Error("A Space must be active to import files");
    const labels = {
      title: i18n.global.t("spaces.importExport.progress.resource"),
      preparing: i18n.global.t("spaces.importExport.progress.preparing"),
    };
    const outcomes = await importSpaceBatch({
      context: spaceImportContext(options.store),
      spaceId,
      requests,
      origin,
      labels,
    });
    const failures = outcomes.filter((outcome) => outcome.status !== "imported");
    if (failures.length) {
      if (failures.length === 1) {
        const [failure] = failures;
        throw failure!.status === "failed" ? failure!.error : new Error("Space import was cancelled");
      }
      throw new AggregateError(
        failures.map((outcome) =>
          outcome.status === "failed" ? outcome.error : new Error(`Import cancelled: ${String(outcome.key)}`),
        ),
        "One or more canvas files could not be imported",
      );
    }
    return outcomes;
  }

  onMounted(() => {
    void fileDropListener
      .attach(
        listenForFileDrops({
          hover(active, position) {
            fileHover.value = active && isFileDropInside(options.dropTarget(), position);
            fileHoverPosition.value = fileHover.value ? position : undefined;
          },
          async drop(payload) {
            if (!operations.allows("create")) return;
            if (!isFileDropInside(options.dropTarget(), payload.position)) return;
            const editor = options.editorSession.activeEditor.value;
            if (!editor) return;
            await importDroppedFiles(
              payload,
              options.store,
              editor.screenToCanvasPoint(payload.position),
            ).catch(options.reportError);
          },
        }),
      )
      .catch(options.reportError);
  });
  onBeforeUnmount(() => fileDropListener.dispose());

  return {
    fileHover,
    fileHoverPosition,
    insertText,
    insertUrl,
    retryCapture,
    importFiles,
    importJsonCanvas: options.store.importJsonCanvas,
    pasteNodes: pasteClipboard,
  };
}

function isImportedOutcome<Key>(
  outcome: SpaceImportBatchOutcome<Key>,
): outcome is Extract<SpaceImportBatchOutcome<Key>, { status: "imported" }> {
  return outcome.status === "imported";
}
