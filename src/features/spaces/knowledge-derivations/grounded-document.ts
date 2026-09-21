import { Channel, invoke } from "@tauri-apps/api/core";

import { aiExecutionTargetPayload, loadAiExecutionTarget } from "../../../foundation/ai/ai-execution";
import type { ExecutionTarget } from "../../../foundation/ai/ai-models";
import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { useCanvasEditorSession } from "../canvas/editor-session";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceMarkdownPayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import type { useSpaceStore } from "../session/space-store";
import { applyDerivationOutput } from "./apply-output";
import type { GroundedDocumentSource } from "./catalog/grounded-document";
import { derivedNodePosition } from "./placement";
import { runDerivation } from "./run-derivation";

type SpaceStore = ReturnType<typeof useSpaceStore>;
export interface CompletedDocument {
  nodeId: string;
  data: SpaceMarkdownPayload;
  relations: SpaceRelation[];
}
export interface GroundedDocumentDraft {
  title: string;
  purpose: string;
  target?: ExecutionTarget | undefined;
}
interface CreateGroundedDocumentContext {
  store: SpaceStore;
  selected: readonly SpaceCanvasNode[];
  sources: readonly GroundedDocumentSource[];
  draft: GroundedDocumentDraft;
}

export async function createGroundedDocument(context: CreateGroundedDocumentContext) {
  const { store, selected, sources, draft } = context;
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const [pending] = await store.addContent(
    { kind: "markdown", title: draft.title.trim(), text: "", derivationState: "pending" },
    derivedNodePosition(selected, useCanvasEditorSession().selection.value.nodePlacements),
    { reveal: true },
  );
  if (!pending) throw new Error("Document output node was not created");
  const jobId = crypto.randomUUID();

  return runDerivation<CompletedDocument>({
    store,
    pendingNode: pending,
    sourceNodeIds: sources.map(({ nodeId }) => nodeId),
    task: {
      id: jobId,
      title: i18n.global.t("spaces.creation.documentTask"),
      detail: draft.title.trim(),
      cancel: () => cancelAiJob(jobId),
    },
    failureMessageKey: "spaces.creation.documentFailed",
    execute: async ({ taskId }) => {
      const stages = new Channel<string>();
      if (taskId) {
        const tasks = useTaskStore();
        // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes its events through onmessage.
        stages.onmessage = (stage) => tasks.detail(taskId, i18n.global.t(`spaces.creation.${stage}`));
      }
      const target = await loadAiExecutionTarget("knowledge", draft.target);
      return invoke<CompletedDocument>("plugin:spaces|create_grounded_document", {
        request: {
          id: jobId,
          spaceId,
          outputNodeId: pending.id,
          inputs: sources.map(({ nodeId, role, sourceRevision }) => ({ nodeId, role, sourceRevision })),
          purpose: draft.purpose.trim(),
          target: aiExecutionTargetPayload(target),
        },
        onStage: stages,
      });
    },
    onSuccess: (completed, currentSpaceId) => {
      applyDerivationOutput(store, { ...completed, spaceId: currentSpaceId });
    },
  });
}
