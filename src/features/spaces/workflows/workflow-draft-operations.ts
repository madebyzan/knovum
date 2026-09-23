import type { WorkflowDraft, WorkflowNodeKind, WorkflowVersion } from "./workflow-client";
import { createWorkflowNode } from "./workflow-node-factory";

export function duplicateWorkflowDraft(draft: WorkflowDraft): WorkflowDraft {
  const copy = structuredClone(draft);
  copy.id = crypto.randomUUID();
  copy.revision = 0;
  copy.createdAt = 0;
  copy.updatedAt = 0;
  copy.definition.title = `${copy.definition.title} Copy`;
  return copy;
}

export function copyWorkflowVersion(spaceId: string, version: WorkflowVersion): WorkflowDraft {
  const now = 0;
  return {
    id: crypto.randomUUID(),
    spaceId,
    revision: 0,
    createdAt: now,
    updatedAt: now,
    definition: {
      ...structuredClone(version.definition),
      title: `${version.definition.title} v${version.version} Copy`,
    },
  };
}

export function addWorkflowNode(draft: WorkflowDraft, kind: WorkflowNodeKind) {
  const id = `${kind}-${draft.definition.nodes.length + 1}`;
  draft.definition.nodes.push(createWorkflowNode(kind, id));
  draft.definition.layout[id] = { x: 160, y: 80 + draft.definition.nodes.length * 72 };
}

export function removeWorkflowNode(draft: WorkflowDraft, nodeId: string) {
  if (new Set(["input", "output"]).has(nodeId)) return;
  draft.definition.nodes = draft.definition.nodes.filter(({ id }) => id !== nodeId);
  draft.definition.edges = draft.definition.edges.filter(
    ({ sourceNodeId, targetNodeId }) => sourceNodeId !== nodeId && targetNodeId !== nodeId,
  );
  delete draft.definition.layout[nodeId];
}

export function connectWorkflowNodes(draft: WorkflowDraft, sourceNodeId: string, targetNodeId: string) {
  if (sourceNodeId === targetNodeId) return;
  const duplicate = draft.definition.edges.some(
    (edge) => edge.sourceNodeId === sourceNodeId && edge.targetNodeId === targetNodeId,
  );
  if (duplicate) return;
  const source = draft.definition.nodes.find(({ id }) => id === sourceNodeId);
  const usedBranches = new Set(
    draft.definition.edges.filter((edge) => edge.sourceNodeId === sourceNodeId).map((edge) => edge.branch),
  );
  const branch =
    source?.kind === "condition"
      ? ((["true", "false"] as const).find((candidate) => !usedBranches.has(candidate)) ?? null)
      : null;
  draft.definition.edges.push({
    id: crypto.randomUUID(),
    sourceNodeId,
    sourcePort: "output",
    targetNodeId,
    targetPort: "input",
    branch,
  });
}
