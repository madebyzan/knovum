import { onBeforeUnmount, onMounted, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { isFileDropInside, listenForFileDrops } from "../../../../foundation/file-drop/events";
import { createAsyncDisposer } from "../../../../foundation/lifecycle/async-disposer";
import type { SpaceImportSourceAccess } from "../../assets/asset-operations";
import { addImportedSpaceAsset } from "../../assets/import";
import { importSpaceBatch, type SpaceImportBatchOutcome } from "../../assets/import-batch";
import { spaceImportContext } from "../../assets/import-context";
import { scheduleSpaceAssetProcessing } from "../../assets/processing-scheduler";
import { pasteSpaceClipboard } from "../../clipboard/paste";
import { useSpaceStore } from "../../session/space-store";
import { captureSpaceUrl } from "../../sources/capture-url";
import { classifySpaceUrl } from "../../sources/classify-url";
import { urlCaptureContext } from "../../sources/url-capture-context";
import type { CanvasFileImportRequest, CanvasFileImportReceipt } from "../contracts/canvas-editor";
import { importDroppedFiles } from "../drop";
import { getCanvasEditorSession } from "../editor-session";
import { pasteSpaceClipboardAt } from "../selection-actions";
import { noteFromPlainText } from "../text-import";
import { useNodeOperationGate } from "../use-node-operation-gate";

export function useSpaceCanvasImports(dropTarget: Readonly<Ref<HTMLElement | null>>) {
  const store = useSpaceStore();
  const editorSession = getCanvasEditorSession();
  const { t } = useI18n();
  const operations = useNodeOperationGate();
  const fileDropListener = createAsyncDisposer();
  const error = shallowRef<string>();
  const fileHover = shallowRef(false);
  const fileHoverPosition = shallowRef<{ x: number; y: number }>();
  watch(
    () => store.activeSpaceId,
    () => (error.value = undefined),
  );

  function reportError(cause: unknown) {
    error.value = getErrorMessage(cause);
  }

  async function insertText(text: string, point: { x: number; y: number }) {
    if (!operations.allows("create")) return;
    const source = classifySpaceUrl(text.trim());
    if (source) return insertUrl(source.url, point);
    await store.addContent(noteFromPlainText(text, t("spaces.editor.note.new")), point);
  }

  async function insertUrl(url: string, point: { x: number; y: number }) {
    if (!operations.allows("create")) return;
    void captureSpaceUrl(urlCaptureContext(), url, point).catch(reportError);
  }

  async function pasteClipboard(point: { x: number; y: number }) {
    if (!operations.allows("create")) return false;
    const spaceId = store.activeSpaceId;
    if (!spaceId) throw new Error("A Space must be active to paste canvas content");
    return pasteSpaceClipboard({
      imageName: t("spaces.nodeTypes.image"),
      pasteNodes: (html) => pasteSpaceClipboardAt(point, html),
      importFiles: async (paths) => {
        await importBatch(
          paths.map((path, key) => ({ key, path, sourceAccess: "clipboard" as const })),
          point,
        );
      },
      insertText: (text) => insertText(text, point),
      addImage: async (image) => {
        const nodes = await addImportedSpaceAsset(image, point, (data, position) => {
          if (store.activeSpaceId !== spaceId) throw new Error("Canvas paste target changed");
          return store.addContent(data, position);
        });
        for (const node of nodes) {
          scheduleSpaceAssetProcessing({ context: store, spaceId, nodeId: node.id, data: node.data });
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
    const spaceId = store.activeSpaceId;
    if (!spaceId) throw new Error("A Space must be active to import files");
    const labels = {
      title: t("spaces.importExport.progress.resource"),
      preparing: t("spaces.importExport.progress.preparing"),
    };
    const outcomes = await importSpaceBatch({
      context: spaceImportContext(),
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
            fileHover.value =
              active && operations.allows("create") && isFileDropInside(dropTarget.value, position);
            fileHoverPosition.value = fileHover.value ? position : undefined;
          },
          async drop(payload) {
            if (!operations.allows("create")) return;
            if (!isFileDropInside(dropTarget.value, payload.position)) return;
            const editor = editorSession.activeEditor.value;
            if (!editor) return;
            await importDroppedFiles(payload, editor.screenToCanvasPoint(payload.position)).catch(
              reportError,
            );
          },
        }),
      )
      .catch(reportError);
  });
  onBeforeUnmount(() => fileDropListener.dispose());

  return {
    error,
    fileHover,
    fileHoverPosition,
    importFiles,
    pasteNodes: pasteClipboard,
  };
}

function isImportedOutcome<Key>(
  outcome: SpaceImportBatchOutcome<Key>,
): outcome is Extract<SpaceImportBatchOutcome<Key>, { status: "imported" }> {
  return outcome.status === "imported";
}
