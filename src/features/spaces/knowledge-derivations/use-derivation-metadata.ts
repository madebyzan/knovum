import { shallowRef, watch, type Ref } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { loadGroundedDocumentMetadata, type GroundedDocumentMetadata } from "./document/metadata";
import { loadDerivationFailure, type DerivationFailure } from "./failure-metadata";
import { loadVisualizationMetadata, type VisualizationMetadata } from "./image/metadata";
import { loadGeneratedVideoMetadata, type GeneratedVideoMetadata } from "./video/metadata";

export function useDerivationMetadata(node: Ref<SpaceCanvasNode | undefined>) {
  const documentMetadata = shallowRef<GroundedDocumentMetadata | null>(null);
  const imageMetadata = shallowRef<VisualizationMetadata | null>(null);
  const videoMetadata = shallowRef<GeneratedVideoMetadata | null>(null);
  const failure = shallowRef<DerivationFailure | null>(null);
  let request = 0;

  watch(
    node,
    async (current) => {
      const currentRequest = ++request;
      documentMetadata.value = null;
      imageMetadata.value = null;
      videoMetadata.value = null;
      failure.value = null;
      if (!current) return;
      const [metadata, failedMetadata] = await Promise.all([
        ["markdown", "image", "video"].includes(current.data.kind) ? load(current) : null,
        isFailedDerivation(current) ? loadDerivationFailure(current.id) : null,
      ]);
      if (currentRequest !== request) return;
      failure.value = failedMetadata;
      if (current.data.kind === "markdown")
        documentMetadata.value = metadata as GroundedDocumentMetadata | null;
      if (current.data.kind === "image") imageMetadata.value = metadata as VisualizationMetadata | null;
      if (current.data.kind === "video") videoMetadata.value = metadata as GeneratedVideoMetadata | null;
    },
    { immediate: true },
  );

  return { documentMetadata, imageMetadata, videoMetadata, failure };
}

function isFailedDerivation(node: SpaceCanvasNode) {
  const processingFailed = "processingState" in node.data && node.data.processingState === "failed";
  const documentFailed = node.data.kind === "markdown" && node.data.derivationState === "failed";
  return processingFailed || documentFailed;
}

function load(node: SpaceCanvasNode) {
  if (node.data.kind === "markdown") return loadGroundedDocumentMetadata(node.id);
  if (node.data.kind === "image") return loadVisualizationMetadata(node.id);
  return loadGeneratedVideoMetadata(node.id);
}
