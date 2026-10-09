import { cloneDeep } from "es-toolkit";
import { shallowRef, type Ref } from "vue";

import type { AgentExtensionSelection } from "../../../shared/contracts/generated/AgentExtensionSelection";
import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";
import type { SubmitConversationTaskRequest } from "../contracts/generated/SubmitConversationTaskRequest";
import type { useConversationDraftStore } from "./conversation-draft-store";

function sameAttachments(left: readonly ConversationAttachment[], right: readonly ConversationAttachment[]) {
  return (
    left.length === right.length &&
    left.every((item) =>
      right.some(
        (attachment) =>
          attachment.kind === item.kind &&
          (attachment.kind === "node" && item.kind === "node"
            ? attachment.nodeId === item.nodeId
            : attachment.kind === "file" && item.kind === "file" && attachment.path === item.path) &&
          attachment.sourceRevision === item.sourceRevision &&
          (attachment.kind === "file" || item.kind === "file" || attachment.role === item.role),
      ),
    )
  );
}

function matchesRequest(
  prompt: string,
  attachments: readonly ConversationAttachment[],
  request: SubmitConversationTaskRequest,
) {
  return prompt.trim() === request.prompt && sameAttachments(attachments, request.attachments);
}

interface ConversationDraftSessionInput {
  drafts: Pick<ReturnType<typeof useConversationDraftStore>, "read" | "load" | "save" | "move" | "remove">;
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  prompt: Ref<string>;
  attachments: Ref<ConversationAttachment[]>;
  extensions: Ref<AgentExtensionSelection>;
  reacquireFileNames: Ref<string[]>;
}

export function createConversationDraftSession(input: ConversationDraftSessionInput) {
  const { drafts } = input;
  const restoring = shallowRef(false);
  let owner: { spaceId: string; conversationId: string } | undefined;
  let restoreGeneration = 0;

  function isCurrentRestore(generation: number, spaceId: string | undefined, conversationId: string) {
    return (
      generation === restoreGeneration &&
      input.spaceId.value === spaceId &&
      input.selectedConversationId.value === conversationId
    );
  }

  function applyRestoredDraft(
    spaceId: string | undefined,
    conversationId: string,
    draft: ReturnType<typeof drafts.read>,
  ) {
    input.prompt.value = draft?.prompt ?? "";
    input.attachments.value = draft?.attachments ?? [];
    input.reacquireFileNames.value = draft?.reacquireFileNames ?? [];
    if (draft) input.extensions.value = cloneDeep(draft.extensions);
    if (spaceId) owner = { spaceId, conversationId };
  }

  function isActiveRequest(spaceId: string, conversationId: string) {
    return input.spaceId.value === spaceId && input.selectedConversationId.value === conversationId;
  }

  async function remember() {
    const current = owner;
    if (!current) return;
    const snapshot = {
      prompt: input.prompt.value,
      attachments: cloneDeep(input.attachments.value),
      extensions: cloneDeep(input.extensions.value),
      reacquireFileNames: [...input.reacquireFileNames.value],
    };
    await drafts.save(current.spaceId, current.conversationId, snapshot);
  }

  async function restore(conversationId: string, requestedSpaceId = input.spaceId.value) {
    const generation = ++restoreGeneration;
    owner = undefined;
    restoring.value = true;
    try {
      const draft = requestedSpaceId ? await drafts.load(requestedSpaceId, conversationId) : undefined;
      if (!isCurrentRestore(generation, requestedSpaceId, conversationId)) return false;
      applyRestoredDraft(requestedSpaceId, conversationId, draft);
      return Boolean(draft);
    } finally {
      if (generation === restoreGeneration) restoring.value = false;
    }
  }

  async function move(fromConversationId: string, toConversationId: string) {
    if (!input.spaceId.value) return;
    await drafts.move(input.spaceId.value, fromConversationId, toConversationId);
  }

  async function clearSubmitted(request: SubmitConversationTaskRequest) {
    const conversationId = request.conversationId ?? "";
    const stored = drafts.read(request.spaceId, conversationId);
    const currentMatches =
      isActiveRequest(request.spaceId, conversationId) &&
      matchesRequest(input.prompt.value, input.attachments.value, request);
    const storedMatches = stored && matchesRequest(stored.prompt, stored.attachments, request);
    if (currentMatches || storedMatches) await drafts.remove(request.spaceId, conversationId);
    if (!isActiveRequest(request.spaceId, conversationId) || !currentMatches) return;
    input.prompt.value = "";
    input.attachments.value = [];
    input.reacquireFileNames.value = [];
  }

  async function restoreSubmitted(request: SubmitConversationTaskRequest) {
    const conversationId = request.conversationId ?? "";
    if (input.spaceId.value === request.spaceId && input.selectedConversationId.value === conversationId) {
      if (!input.prompt.value && !input.attachments.value.length) {
        input.prompt.value = request.prompt;
        input.attachments.value = [...(request.attachments ?? [])];
        input.extensions.value = cloneDeep(request.extensions);
        input.reacquireFileNames.value = [];
        await drafts.save(request.spaceId, conversationId, {
          prompt: request.prompt,
          attachments: [...(request.attachments ?? [])],
          extensions: cloneDeep(request.extensions),
          reacquireFileNames: [],
        });
      }
      return;
    }
    if (!drafts.read(request.spaceId, conversationId)) {
      await drafts.save(request.spaceId, conversationId, {
        prompt: request.prompt,
        attachments: [...(request.attachments ?? [])],
        extensions: cloneDeep(request.extensions),
        reacquireFileNames: [],
      });
    }
  }

  function discardFileReference(name: string) {
    input.reacquireFileNames.value = input.reacquireFileNames.value.filter((candidate) => candidate !== name);
  }

  function invalidate() {
    restoreGeneration += 1;
    owner = undefined;
  }

  return {
    restoring,
    ownerEpoch: () => restoreGeneration,
    remember,
    restore,
    move,
    clearSubmitted,
    restoreSubmitted,
    discardFileReference,
    invalidate,
  };
}
