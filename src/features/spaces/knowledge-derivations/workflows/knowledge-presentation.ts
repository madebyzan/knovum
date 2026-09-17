import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import { i18n } from "../../../../foundation/i18n/i18n";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import { ensureStructuredDocument } from "../document/structured-input";
import { executePersistedWorkflow } from "../orchestration/persisted-workflow";
import { loadWorkflow } from "../orchestration/workflow-registry";
import { createPresentationFromDocument, presentationInputs } from "../slide-deck/slide-deck";
import { nodeStepOutput } from "./output";

type Store = ReturnType<typeof useSpaceStore>;
type Action = "structure-document" | "render-presentation";

interface KnowledgePresentationOptions {
  store: Store;
  nodes: readonly SpaceCanvasNode[];
  sources: readonly GroundedDocumentSource[];
  draft: KnowledgePresentationDraft;
}

export interface KnowledgePresentationDraft {
  title: string;
  purpose: string;
  textTarget?: ExecutionTarget | undefined;
}

export async function createPresentationFromKnowledge(options: KnowledgePresentationOptions) {
  const spaceId = options.store.activeSpaceId;
  if (!spaceId) return;
  const workflow = await loadWorkflow<Action>("knowledge-presentation");
  return executePersistedWorkflow<Action, SpaceCanvasNode>({
    spaceId,
    workflow,
    task: { title: i18n.global.t("spaces.creation.presentationTask") },
    adapters: {
      "structure-document": async () => {
        const document = await ensureStructuredDocument({
          store: options.store,
          selected: options.nodes,
          sources: options.sources,
          title: options.draft.title,
          purpose: options.draft.purpose,
          target: options.draft.textTarget,
        });
        if (!document) throw new Error("Presentation document was not created");
        return nodeStepOutput(document);
      },
      "render-presentation": async ({ inputs: stepInputs }) => {
        const document = stepInputs.document?.output;
        if (!document) throw new Error("Presentation document output is unavailable");
        const images = options.nodes.filter((node) => node.data.kind === "image");
        const inputs = presentationInputs([document, ...images]);
        if (!inputs) throw new Error("Presentation inputs are invalid");
        const placementNodes = options.nodes.some(({ id }) => id === document.id)
          ? options.nodes
          : [...options.nodes, document];
        const output = await createPresentationFromDocument({
          store: options.store,
          nodes: placementNodes,
          inputs,
        });
        if (!output) throw new Error("Presentation output was not created");
        return nodeStepOutput(output);
      },
    },
  });
}
