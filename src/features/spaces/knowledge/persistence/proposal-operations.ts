import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { KnowledgeProposal } from "../../contracts/generated/KnowledgeProposal";
import type { ProposalAcceptance } from "../../contracts/generated/ProposalAcceptance";
import type { ProposalOrigin } from "../../contracts/generated/ProposalOrigin";

export function listKnowledgeProposals(
  spaceId: string,
  origin: ProposalOrigin | null,
): Promise<KnowledgeProposal[]> {
  return invokeCommand("plugin:spaces|list_knowledge_proposals", { spaceId, origin });
}

/** Accepts the proposal, or the chosen changes; the backend keeps only the part that stands on its own. */
export function acceptKnowledgeProposal(
  proposalId: string,
  changes: readonly number[] | null,
): Promise<ProposalAcceptance> {
  return invokeCommand("plugin:spaces|accept_knowledge_proposal", { proposalId, changes });
}

export function dismissKnowledgeProposal(proposalId: string): Promise<void> {
  return invokeCommand("plugin:spaces|dismiss_knowledge_proposal", { proposalId });
}
