import { i18n } from "../../../../foundation/i18n/i18n";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import { ensureStructuredDocument } from "../document/structured-input";
import { executePersistedWorkflow } from "../orchestration/persisted-workflow";
import { loadWorkflow } from "../orchestration/workflow-registry";
import {
  createKnowledgeVideoFromDocument,
  knowledgeVideoInputs,
  type KnowledgeVideoDraft,
} from "../video/knowledge-video";
import { nodeStepOutput } from "./output";

type Store = ReturnType<typeof useSpaceStore>;
type Action = "write-script" | "generate-video";

interface KnowledgeVideoWorkflowOptions {
  store: Store;
  nodes: readonly SpaceCanvasNode[];
  sources: readonly GroundedDocumentSource[];
  draft: KnowledgeVideoDraft;
}

export async function createKnowledgeVideoFromKnowledge(options: KnowledgeVideoWorkflowOptions) {
  const spaceId = options.store.activeSpaceId;
  if (!spaceId) return;
  const workflow = await loadWorkflow<Action>("knowledge-video");
  return executePersistedWorkflow<Action, SpaceCanvasNode>({
    spaceId,
    workflow,
    task: { title: i18n.global.t("spaces.creation.videoTask"), detail: options.draft.title },
    adapters: {
      "write-script": async () => {
        const script = await ensureStructuredDocument({
          store: options.store,
          selected: options.nodes,
          sources: options.sources,
          title: i18n.global.t("spaces.creation.videoScriptTitle"),
          purpose: i18n.global.t("spaces.creation.videoScriptPurpose"),
          reuseExisting: false,
          target: options.draft.textTarget,
        });
        if (!script) throw new Error("Video script was not created");
        return nodeStepOutput(script);
      },
      "generate-video": async ({ inputs: stepInputs }) => {
        const script = stepInputs.script?.output;
        if (!script) throw new Error("Video script output is unavailable");
        const images = options.nodes.filter((node) => node.data.kind === "image");
        const inputs = knowledgeVideoInputs([script, ...images]);
        if (!inputs) throw new Error("Knowledge video inputs are invalid");
        const output = await createKnowledgeVideoFromDocument({
          store: options.store,
          inputs,
          draft: options.draft,
        });
        if (!output) throw new Error("Knowledge video output was not created");
        return nodeStepOutput(output);
      },
    },
  });
}
