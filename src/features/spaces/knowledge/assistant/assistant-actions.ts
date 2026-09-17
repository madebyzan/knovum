import { toast } from "vue-sonner";

import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import { cancelAiJob, decodeAiErrorPayload } from "../../../../foundation/ai/ai-runtime";
import { i18n } from "../../../../foundation/i18n/i18n";
import type { KnowledgeAssistantActionContext } from "./action-context";
import {
  appendKnowledgeMessages,
  cacheKnowledgeThreads,
  knowledgeAssistantQueryKeys,
} from "./assistant-query";
import { askGroundedKnowledge } from "./grounded-assistant";
import { deleteKnowledgeThread, renameKnowledgeThread } from "./thread-actions";
import type { PendingAsk } from "./use-assistant-session";

const t = i18n.global.t;

export async function askKnowledge(context: KnowledgeAssistantActionContext, value?: string) {
  const request = prepareKnowledgeRequest(context, value);
  if (!request) return;
  const { spaceId, prompt, target } = request;
  const generation = context.generation.value;
  const token: PendingAsk = { generation, spaceId, jobId: crypto.randomUUID() };
  context.pendingAsk.value = token;
  context.pendingQuestion.value = prompt;
  context.question.value = "";
  context.streamedAnswer.value = "";
  context.answerStage.value = "retrieving";
  try {
    const answer = await requestKnowledgeAnswer({ context, spaceId, prompt, target, token });
    await cacheKnowledgeAnswer(context, spaceId, answer);
    if (isCurrentConversation(context, generation, spaceId)) applyKnowledgeAnswer(context, answer);
  } catch (error) {
    if (isCurrentConversation(context, generation, spaceId)) reportKnowledgeAnswerFailure(error);
  } finally {
    if (context.pendingAsk.value === token) {
      context.pendingAsk.value = undefined;
      context.pendingQuestion.value = "";
      context.streamedAnswer.value = "";
      context.answerStage.value = undefined;
    }
  }
}

function prepareKnowledgeRequest(context: KnowledgeAssistantActionContext, value?: string) {
  const spaceId = context.store.activeSpace?.id;
  const prompt = (value ?? context.question.value).trim();
  const target = context.executionTarget.value;
  if (!spaceId || !target || !prompt || context.busy.value) return;
  return { spaceId, prompt, target };
}

export async function cancelKnowledgeAsk(context: KnowledgeAssistantActionContext) {
  const jobId = context.pendingAsk.value?.jobId;
  if (!jobId) return;
  try {
    await cancelAiJob(jobId);
  } catch (error) {
    reportKnowledgeAnswerFailure(error);
  }
}

// Stopping an answer is a normal user outcome, not a failure worth a toast.
function reportKnowledgeAnswerFailure(error: unknown) {
  const decoded = decodeAiErrorPayload(error);
  if (decoded?.code === "cancelled") return;
  toast.error(decoded?.message ?? String(error));
}

interface KnowledgeAnswerRequest {
  context: KnowledgeAssistantActionContext;
  spaceId: string;
  prompt: string;
  target: ExecutionTarget;
  token: PendingAsk;
}

function requestKnowledgeAnswer({ context, spaceId, prompt, target, token }: KnowledgeAnswerRequest) {
  const threadId = context.threadId.value;
  return askGroundedKnowledge({
    jobId: token.jobId,
    spaceId,
    scope: context.activeScope.value,
    nodeIds: context.activeIds.value,
    question: prompt,
    target,
    onStage: (stage) => {
      if (context.pendingAsk.value === token) context.answerStage.value = stage;
    },
    onDelta: (text) => {
      if (context.pendingAsk.value === token) context.streamedAnswer.value += text;
    },
    ...(threadId === undefined ? {} : { threadId }),
  });
}

async function cacheKnowledgeAnswer(
  context: KnowledgeAssistantActionContext,
  spaceId: string,
  answer: Awaited<ReturnType<typeof askGroundedKnowledge>>,
) {
  await context.queryClient.cancelQueries({
    queryKey: knowledgeAssistantQueryKeys.messages(answer.threadId),
    exact: true,
  });
  appendKnowledgeMessages(context.queryClient, answer.threadId, [answer.user, answer.assistant]);
  void context.queryClient.invalidateQueries({ queryKey: knowledgeAssistantQueryKeys.threads(spaceId) });
}

function applyKnowledgeAnswer(
  context: KnowledgeAssistantActionContext,
  answer: Awaited<ReturnType<typeof askGroundedKnowledge>>,
) {
  context.threadId.value = answer.threadId;
  context.semantic.value = answer.semantic;
}

export async function renameThread(context: KnowledgeAssistantActionContext, title: string) {
  const id = context.threadId.value;
  const spaceId = context.store.activeSpace?.id;
  if (!id || !spaceId) return;
  try {
    const next = await renameKnowledgeThread(id, spaceId, title);
    await context.queryClient.cancelQueries({
      queryKey: knowledgeAssistantQueryKeys.threads(spaceId),
      exact: true,
    });
    cacheKnowledgeThreads(context.queryClient, spaceId, next);
  } catch (error) {
    if (context.store.activeSpace?.id === spaceId) toast.error(String(error));
  }
}

export async function deleteThread(context: KnowledgeAssistantActionContext) {
  const id = context.threadId.value;
  const spaceId = context.store.activeSpace?.id;
  const generation = context.generation.value;
  if (!id || !spaceId) return;
  try {
    const next = await deleteKnowledgeThread(id, spaceId, {
      title: t("spaces.assistant.deleteKnowledgeThread"),
      description: t("spaces.assistant.deleteKnowledgeThreadDescription"),
    });
    if (!next) return;
    await context.queryClient.cancelQueries({
      queryKey: knowledgeAssistantQueryKeys.threads(spaceId),
      exact: true,
    });
    cacheKnowledgeThreads(context.queryClient, spaceId, next);
    context.queryClient.removeQueries({ queryKey: knowledgeAssistantQueryKeys.messages(id), exact: true });
    if (isCurrentConversation(context, generation, spaceId) && context.threadId.value === id)
      context.resetThread();
  } catch (error) {
    if (context.store.activeSpace?.id === spaceId) toast.error(String(error));
  }
}

function isCurrentConversation(
  context: KnowledgeAssistantActionContext,
  generation: number,
  spaceId: string,
) {
  return context.generation.value === generation && context.store.activeSpace?.id === spaceId;
}
