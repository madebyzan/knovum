export interface DerivationSource {
  nodeId: string | null;
  sourceIdentity: string;
  title: string;
  role: string;
  sourceRevision: string;
  revisionBasis: "asset" | "rendered" | "knowledge";
  inputRevision: string;
  current: boolean;
}
