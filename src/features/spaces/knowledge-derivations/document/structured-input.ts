import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import { createGroundedDocument } from "../grounded-document";
import { isPopulatedMarkdown } from "./export";

type Store = ReturnType<typeof useSpaceStore>;

export interface EnsureStructuredDocumentOptions {
  store: Store;
  selected: readonly SpaceCanvasNode[];
  sources: readonly GroundedDocumentSource[];
  title: string;
  purpose: string;
  reuseExisting?: boolean;
  target?: ExecutionTarget | undefined;
}

export async function ensureStructuredDocument(options: EnsureStructuredDocumentOptions) {
  const existing = options.reuseExisting === false ? undefined : options.selected.find(isPopulatedMarkdown);
  if (existing) return existing;
  const completed = await createGroundedDocument({
    store: options.store,
    selected: options.selected,
    sources: options.sources,
    draft: { title: options.title, purpose: options.purpose, target: options.target },
  });
  if (!completed) return;
  const output = options.store.nodes.find((node) => node.id === completed.nodeId);
  return output && isPopulatedMarkdown(output) ? output : undefined;
}
