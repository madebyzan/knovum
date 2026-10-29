import { useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";

import { queryClient } from "../../../../foundation/query/query-client";
import type { StatementSupport } from "../../contracts/generated/StatementSupport";
import {
  correctStatementSupport,
  loadAnswerVerification,
  recordAnswerClaims,
  verifyAnswer,
} from "../persistence/verification-operations";
import { SPACE_KNOWLEDGE_QUERY_KEYS } from "../queries/query-options";
import { useInvalidatingEvent } from "../queries/use-invalidating-event";
import { statementViews, tally } from "./answer-verification-view";

/** The verification of one stored answer, kept current while the answer is shown. */
export function useAnswerVerification(
  spaceId: MaybeRefOrGetter<string>,
  messageId: MaybeRefOrGetter<string>,
) {
  const key = () => SPACE_KNOWLEDGE_QUERY_KEYS.answerVerification(toValue(messageId));
  const query = useQuery(
    computed(() => ({ queryKey: key(), queryFn: () => loadAnswerVerification(toValue(messageId)) })),
  );
  useInvalidatingEvent<{ messageId: string }>("space-answer-verification-update", (payload) =>
    payload.messageId === toValue(messageId) ? key() : null,
  );
  const verification = computed(() => query.data.value ?? null);
  const views = computed(() => (verification.value ? statementViews(verification.value) : []));
  const counts = computed(() => tally(views.value));

  async function changed<T>(write: Promise<T>): Promise<T> {
    const result = await write;
    await queryClient.invalidateQueries({ queryKey: key() });
    return result;
  }

  return {
    verification,
    views,
    counts,
    verify: () => changed(verifyAnswer(toValue(messageId))),
    correct: (statement: number, support: StatementSupport) =>
      changed(correctStatementSupport(toValue(messageId), statement, support)),
    record: async (statements: readonly number[]) => {
      const receipt = await recordAnswerClaims(toValue(messageId), statements);
      await queryClient.invalidateQueries({
        queryKey: SPACE_KNOWLEDGE_QUERY_KEYS.proposals(toValue(spaceId)),
      });
      return receipt;
    },
  };
}
