import { invoke } from "@tauri-apps/api/core";
import { readFile } from "@tauri-apps/plugin-fs";
import { extname } from "pathe";

import { i18n } from "../../../../foundation/i18n/i18n";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceFilePayload, SpaceImagePayload } from "../../model/nodes/node-payload";
import type { useSpaceStore } from "../../session/space-store";
import { markProcessingNodeFailed } from "../pending-node";
import { derivedNodePosition } from "../placement";
import { runDerivation } from "../run-derivation";

type Store = ReturnType<typeof useSpaceStore>;
export interface PresentationInputs {
  document: SpaceCanvasNode & {
    data: { kind: "markdown"; title: string; text: string; knowledgeRevision: string };
  };
  images: Array<SpaceCanvasNode & { data: SpaceImagePayload & { sourceRevision: string } }>;
}
interface Output {
  nodeId: string;
  data: SpaceFilePayload;
}
interface CreatePresentationOptions {
  store: Store;
  nodes: readonly SpaceCanvasNode[];
  inputs: PresentationInputs;
}

export function presentationInputs(nodes: readonly SpaceCanvasNode[]): PresentationInputs | undefined {
  const document = nodes.find(
    (node) =>
      node.data.kind === "markdown" && Boolean(node.data.text) && Boolean(node.data.knowledgeRevision),
  );
  const images = nodes.filter(
    (node): node is PresentationInputs["images"][number] =>
      node.data.kind === "image" && Boolean(node.data.assetPath) && Boolean(node.data.sourceRevision),
  );
  return document ? { document: document as PresentationInputs["document"], images } : undefined;
}

export async function createPresentationFromDocument({ store, nodes, inputs }: CreatePresentationOptions) {
  if (!store.activeSpaceId) return;
  const spaceId = store.activeSpaceId;
  const title = `${inputs.document.data.title}.pptx`;
  const assetPath = await invoke<string>("plugin:spaces|allocate_presentation_pptx_path");
  const [output] = await store.addContent(
    { kind: "file", title, fileName: title, assetPath, processingState: "processing" },
    derivedNodePosition(nodes, useCanvasEditorSession().selection.value.nodePlacements),
    { reveal: true },
  );
  if (!output) throw new Error("Presentation output node was not created");

  const presentation = await runDerivation<Output>({
    store,
    pendingNode: output,
    sourceNodeIds: [inputs.document.id, ...inputs.images.map(({ id }) => id)],
    task: {
      title: i18n.global.t("spaces.creation.presentationTask"),
      detail: title,
    },
    failureMessageKey: "spaces.creation.presentationFailed",
    execute: async () => {
      const bytes = await render(inputs);
      return invoke<Output>("plugin:spaces|submit_presentation", {
        request: {
          id: crypto.randomUUID(),
          spaceId,
          outputNodeId: output.id,
          bytes: [...bytes],
          inputs: presentationInputsForPersistence(inputs),
        },
      });
    },
    onSuccess: async (completed, currentSpaceId) => {
      store.applyNodeDataForSpace(currentSpaceId, completed.nodeId, completed.data);
      await store.refreshActiveSpace();
    },
    onError: async () => {
      await markProcessingNodeFailed(store, spaceId, output);
    },
  });
  return presentation ? store.nodes.find((node) => node.id === presentation.nodeId) : undefined;
}

function presentationInputsForPersistence(inputs: PresentationInputs) {
  return [inputs.document, ...inputs.images].map((node, index) => ({
    nodeId: node.id,
    role: index === 0 ? "primary" : "source",
    sourceRevision: node.data.knowledgeRevision ?? node.data.sourceRevision,
  }));
}

async function render(inputs: PresentationInputs): Promise<Uint8Array> {
  const images = await Promise.all(
    inputs.images.map(async ({ data }) => ({
      title: data.title,
      mime: imageMime(data.fileName),
      bytes: transferableBytes(await readFile(data.assetPath)),
    })),
  );
  const worker = new Worker(new URL("./pptx-worker.ts", import.meta.url), { type: "module" });
  return new Promise((resolve, reject) => {
    let settled = false;
    const finish = (outcome: Uint8Array | Error) => {
      if (settled) return;
      settled = true;
      worker.terminate();
      if (outcome instanceof Error) reject(outcome);
      else resolve(outcome);
    };
    worker.addEventListener(
      "message",
      ({ data }: MessageEvent<{ bytes?: ArrayBuffer; error?: string }>) => {
        finish(data.bytes ? new Uint8Array(data.bytes) : new Error(data.error ?? "PPTX generation failed"));
      },
      { once: true },
    );
    worker.addEventListener(
      "error",
      (event) => {
        finish(new Error(event.message));
      },
      { once: true },
    );
    worker.postMessage(
      { title: inputs.document.data.title, markdown: inputs.document.data.text, images },
      images.map(({ bytes }) => bytes),
    );
  });
}
const extension = (name: string) => extname(name).replace(/^\./u, "").toLowerCase() || "png";
const imageMime = (name: string) => `image/${extension(name) === "jpg" ? "jpeg" : extension(name)}`;
function transferableBytes(bytes: Uint8Array): ArrayBuffer {
  const buffer = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(buffer).set(bytes);
  return buffer;
}
