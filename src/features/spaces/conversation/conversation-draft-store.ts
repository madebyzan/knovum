import { cloneDeep } from "es-toolkit";
import { defineStore } from "pinia";
import { shallowRef } from "vue";

import type { ConversationDraft } from "../contracts/generated/ConversationDraft";
import {
  loadConversationDraft,
  moveConversationDraft,
  removeConversationDraft,
  saveConversationDraft,
} from "./conversation-ipc";

export type ConversationComposerDraft = Omit<ConversationDraft, "spaceId" | "conversationId">;

export const useConversationDraftStore = defineStore("spaces-conversation-drafts", () => {
  const drafts = shallowRef<Record<string, ConversationComposerDraft>>({});
  const saveGenerations = new Map<string, number>();
  let persistenceTail = Promise.resolve<unknown>(undefined);

  function advanceSaveGeneration(key: string) {
    const generation = (saveGenerations.get(key) ?? 0) + 1;
    saveGenerations.set(key, generation);
    return generation;
  }

  function persist<T>(operation: () => Promise<T>) {
    const draftOperation = persistenceTail.then(operation, operation);
    persistenceTail = draftOperation.catch(() => undefined);
    return draftOperation;
  }

  function read(spaceId: string, conversationId: string): ConversationComposerDraft | undefined {
    const draft = drafts.value[`${spaceId}:${conversationId}`];
    return (
      draft && {
        prompt: draft.prompt,
        attachments: [...draft.attachments],
        extensions: cloneDeep(draft.extensions),
        reacquireFileNames: [...draft.reacquireFileNames],
      }
    );
  }

  async function load(spaceId: string, conversationId: string) {
    const persisted = await persist(() => loadConversationDraft(spaceId, conversationId));
    if (!persisted) return undefined;
    const draft: ConversationComposerDraft = {
      prompt: persisted.prompt,
      attachments: persisted.attachments,
      extensions: persisted.extensions,
      reacquireFileNames: persisted.reacquireFileNames,
    };
    saveCached(spaceId, conversationId, draft);
    return read(spaceId, conversationId);
  }

  function saveCached(spaceId: string, conversationId: string, draft: ConversationComposerDraft) {
    drafts.value = {
      ...drafts.value,
      [`${spaceId}:${conversationId}`]: {
        prompt: draft.prompt,
        attachments: [...draft.attachments],
        extensions: cloneDeep(draft.extensions),
        reacquireFileNames: [...draft.reacquireFileNames],
      },
    };
  }

  async function save(spaceId: string, conversationId: string, draft: ConversationComposerDraft) {
    const key = `${spaceId}:${conversationId}`;
    const generation = advanceSaveGeneration(key);
    saveCached(spaceId, conversationId, draft);
    const snapshot = read(spaceId, conversationId);
    if (!snapshot) return;
    const persisted = await persist(() =>
      saveConversationDraft({
        spaceId,
        conversationId,
        ...snapshot,
      }),
    );
    if (saveGenerations.get(key) !== generation) return;
    const current = read(spaceId, conversationId);
    if (current)
      saveCached(spaceId, conversationId, {
        ...current,
        reacquireFileNames: persisted.reacquireFileNames,
      });
  }

  async function move(spaceId: string, fromId: string, toId: string) {
    if (!drafts.value[`${spaceId}:${fromId}`]) return;
    advanceSaveGeneration(`${spaceId}:${fromId}`);
    advanceSaveGeneration(`${spaceId}:${toId}`);
    await persist(() => moveConversationDraft(spaceId, fromId, toId));
    const next = { ...drafts.value };
    const draft = next[`${spaceId}:${fromId}`];
    if (!draft) return;
    delete next[`${spaceId}:${fromId}`];
    next[`${spaceId}:${toId}`] = draft;
    drafts.value = next;
  }

  async function remove(spaceId: string, conversationId: string) {
    advanceSaveGeneration(`${spaceId}:${conversationId}`);
    await persist(() => removeConversationDraft(spaceId, conversationId));
    const next = { ...drafts.value };
    delete next[`${spaceId}:${conversationId}`];
    drafts.value = next;
  }

  return { read, load, save, move, remove };
});
