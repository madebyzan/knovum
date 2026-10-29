import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { changedSources } from "./provenance";

type SpaceKnowledgeHealthIssueKind =
  | "unsupported-claim"
  | "contradicted-claim"
  | "ungrounded-evidence"
  | "answered-open-question"
  | "stale-provenance";

export interface SpaceKnowledgeHealthIssue {
  id: string;
  kind: SpaceKnowledgeHealthIssueKind;
  severity: "info" | "warning" | "critical";
  nodeId: string;
  title: string;
  sourceNodeId?: string;
  sourceRevision?: string;
  currentRevision?: string;
}

/**
 * How many sources of each item have changed since it was kept. The health check reports what was
 * stale when it ran; the sources as they are now settle what still is.
 */
export function sourceUpdateCounts(
  issues: readonly SpaceKnowledgeHealthIssue[],
  sources: ReadonlyMap<string, SpaceNodePayload>,
) {
  const counts = new Map<string, number>();
  const stale = issues.filter(({ kind }) => kind === "stale-provenance");
  for (const { nodeId } of changedSources(stale, sources)) counts.set(nodeId, (counts.get(nodeId) ?? 0) + 1);
  return counts;
}
