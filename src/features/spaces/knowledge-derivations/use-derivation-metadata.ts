import { useQueries } from "@tanstack/vue-query";
import { computed, type Ref } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { loadGroundedDocumentMetadata, type GroundedDocumentMetadata } from "./document/metadata";
import { loadDerivationFailure, type DerivationFailure } from "./failure-metadata";
import { loadVisualizationMetadata, type VisualizationMetadata } from "./image/metadata";
import { loadGeneratedVideoMetadata, type GeneratedVideoMetadata } from "./video/metadata";

type DerivationMetadata =
  | { kind: "image"; value: VisualizationMetadata | null }
  | { kind: "markdown"; value: GroundedDocumentMetadata | null }
  | { kind: "video"; value: GeneratedVideoMetadata | null };

type DerivationQuery = {
  enabled: boolean;
  queryFn: () => Promise<DerivationMetadata | null>;
  queryKey: (string | undefined)[];
};

type FailureQuery = {
  enabled: boolean;
  queryFn: () => Promise<DerivationFailure | null>;
  queryKey: (string | undefined)[];
};

const METADATA_KINDS = new Set<string>(["markdown", "image", "video"]);

export function useDerivationMetadata(node: Ref<SpaceCanvasNode | undefined>) {
  const results = useQueries({
    queries: computed((): [DerivationQuery, FailureQuery] => derivationQueries(node.value)),
    combine: ([metadata, failure]) => {
      const loaded = metadata?.data;
      return {
        documentMetadata: loaded?.kind === "markdown" ? loaded.value : null,
        imageMetadata: loaded?.kind === "image" ? loaded.value : null,
        videoMetadata: loaded?.kind === "video" ? loaded.value : null,
        failure: failure?.data ?? null,
      };
    },
  });

  return {
    documentMetadata: computed(() => results.value.documentMetadata),
    imageMetadata: computed(() => results.value.imageMetadata),
    videoMetadata: computed(() => results.value.videoMetadata),
    failure: computed(() => results.value.failure),
  };
}

/** Derivation state joins the key so a run that finishes or fails refetches instead of going stale. */
function derivationQueries(node: SpaceCanvasNode | undefined): [DerivationQuery, FailureQuery] {
  const id = node?.id;
  const scope = node
    ? [node.data.kind, processingState(node), derivationState(node)]
    : ["none", undefined, undefined];
  return [
    {
      queryKey: ["spaces", id, "derivation-metadata", ...scope],
      queryFn: async () => (node ? loadMetadata(node) : null),
      enabled: node !== undefined && METADATA_KINDS.has(node.data.kind),
    },
    {
      queryKey: ["spaces", id, "derivation-failure", ...scope],
      queryFn: async () => (node ? loadDerivationFailure(node.id) : null),
      enabled: node !== undefined && isFailedDerivation(node),
    },
  ];
}

function processingState(node: SpaceCanvasNode) {
  return "processingState" in node.data ? node.data.processingState : undefined;
}

function derivationState(node: SpaceCanvasNode) {
  return node.data.kind === "markdown" ? node.data.derivationState : undefined;
}

function isFailedDerivation(node: SpaceCanvasNode) {
  return processingState(node) === "failed" || derivationState(node) === "failed";
}

async function loadMetadata(node: SpaceCanvasNode): Promise<DerivationMetadata> {
  if (node.data.kind === "markdown")
    return { kind: "markdown", value: await loadGroundedDocumentMetadata(node.id) };
  if (node.data.kind === "image") return { kind: "image", value: await loadVisualizationMetadata(node.id) };
  return { kind: "video", value: await loadGeneratedVideoMetadata(node.id) };
}
