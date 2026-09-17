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
