import { toValue, type MaybeRefOrGetter } from "vue";

import { queryClient } from "../../../../foundation/query/query-client";
import type { ProposalOrigin } from "../../contracts/generated/ProposalOrigin";
import {
  acceptKnowledgeProposal,
  dismissKnowledgeProposal,
  listKnowledgeProposals,
} from "../persistence/proposal-operations";
import { SPACE_KNOWLEDGE_QUERY_KEYS } from "../queries/query-options";
import { useInvalidatingEvent } from "../queries/use-invalidating-event";
import { useSpaceQuery } from "../queries/use-space-query";

/** The pending Knowledge Proposals of a Space; with an origin, only those it produced. */
export function useKnowledgeProposals(
  spaceId: MaybeRefOrGetter<string>,
  origin: MaybeRefOrGetter<ProposalOrigin | null>,
) {
  const query = useSpaceQuery(
    spaceId,
    (id) => [...SPACE_KNOWLEDGE_QUERY_KEYS.proposals(id), toValue(origin)],
    (id) => listKnowledgeProposals(id, toValue(origin)),
  );
  useInvalidatingEvent<{ spaceId: string }>("space-knowledge-proposals-update", (payload) =>
    payload.spaceId === toValue(spaceId) ? SPACE_KNOWLEDGE_QUERY_KEYS.proposals(payload.spaceId) : null,
  );

  /** Accepting writes knowledge, so every Knowledge query of the Space is read again. */
  async function accept(proposalId: string, changes: readonly number[] | null) {
    const acceptance = await acceptKnowledgeProposal(proposalId, changes);
    await queryClient.invalidateQueries({ queryKey: SPACE_KNOWLEDGE_QUERY_KEYS.space(toValue(spaceId)) });
    return acceptance;
  }

  async function dismiss(proposalId: string) {
    await dismissKnowledgeProposal(proposalId);
    await queryClient.invalidateQueries({ queryKey: SPACE_KNOWLEDGE_QUERY_KEYS.proposals(toValue(spaceId)) });
  }

  return { proposals: query.data, accept, dismiss };
}
