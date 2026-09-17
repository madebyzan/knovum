export interface DerivationSource {
  nodeId: string | null;
  sourceIdentity: string;
  title: string;
  role: string;
  sourceRevision: string;
  current: boolean;
}
