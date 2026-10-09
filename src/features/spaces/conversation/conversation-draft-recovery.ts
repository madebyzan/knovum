import type { SubmitConversationTaskRequest } from "../contracts/generated/SubmitConversationTaskRequest";

export async function restoreConversationDraftAfterFailure(
  restore: (request: SubmitConversationTaskRequest) => Promise<void>,
  request: SubmitConversationTaskRequest,
  failure: unknown,
) {
  try {
    await restore(request);
    return failure;
  } catch (restorationError) {
    return new AggregateError([failure, restorationError], "Conversation draft restoration failed");
  }
}
