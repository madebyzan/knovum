import type { WorkflowDefinition, WorkflowNode } from "./workflow-client";

export interface WorkflowVersionDiff {
  addedNodes: string[];
  removedNodes: string[];
  changedNodes: string[];
  addedEdges: string[];
  removedEdges: string[];
  policyChanges: string[];
}

export function diffWorkflowVersions(
  previous: WorkflowDefinition,
  current: WorkflowDefinition,
): WorkflowVersionDiff {
  const previousNodes = new Map(previous.nodes.map((node) => [node.id, node]));
  const currentNodes = new Map(current.nodes.map((node) => [node.id, node]));
  const addedNodes = current.nodes.filter(({ id }) => !previousNodes.has(id)).map(nodeLabel);
  const removedNodes = previous.nodes.filter(({ id }) => !currentNodes.has(id)).map(nodeLabel);
  const changedNodes = current.nodes
    .filter((node) => previousNodes.has(node.id) && stable(previousNodes.get(node.id)) !== stable(node))
    .map(nodeLabel);
  const previousEdges = new Set(previous.edges.map(edgeLabel));
  const currentEdges = new Set(current.edges.map(edgeLabel));
  const policyChanges = [
    previous.failurePolicy === current.failurePolicy
      ? undefined
      : `failurePolicy: ${previous.failurePolicy} → ${current.failurePolicy}`,
    stable(previous.limits) === stable(current.limits) ? undefined : "limits",
  ].filter((value): value is string => Boolean(value));
  return {
    addedNodes,
    removedNodes,
    changedNodes,
    addedEdges: [...currentEdges].filter((edge) => !previousEdges.has(edge)),
    removedEdges: [...previousEdges].filter((edge) => !currentEdges.has(edge)),
    policyChanges,
  };
}

function nodeLabel(node: WorkflowNode) {
  return `${node.id} · ${node.kind}`;
}

function edgeLabel(edge: WorkflowDefinition["edges"][number]) {
  const branch = edge.branch ? ` [${edge.branch}]` : "";
  return `${edge.sourceNodeId}.${edge.sourcePort} → ${edge.targetNodeId}.${edge.targetPort}${branch}`;
}

function stable(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(stable).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.entries(value)
      .toSorted(([left], [right]) => left.localeCompare(right))
      .map(([key, entry]) => `${key}:${stable(entry)}`)
      .join(",")}}`;
  }
  return JSON.stringify(value) ?? "undefined";
}
