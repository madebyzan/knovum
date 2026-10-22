import { cloneDeep } from "es-toolkit";
import { defineStore } from "pinia";
import { shallowRef } from "vue";

import type { ConversationDraft } from "../contracts/generated/ConversationDraft";
import { useSpaceStore } from "../session/space-store";
import {
  loadConversationDraft,
  moveConversationDraft,
  removeConversationDraft,
  saveConversationDraft,
} from "./conversation-ipc";

export type ConversationComposerDraft = Omit<ConversationDraft, "spaceId" | "conversationId">;

/** Composer drafts per Space; an unsaved Space keeps its drafts in memory until it is created. */
export const useConversationDraftStore = defineStore("spaces-conversation-drafts", () => {
  const spaces = useSpaceStore();
  const draftByConversationKey = shallowRef<Record<string, ConversationComposerDraft>>({});
  const saveGenerationByConversationKey = new Map<string, number>();
  let persistenceTail = Promise.resolve<unknown>(undefined);

  function advanceSaveGeneration(key: string) {
    const generation = (saveGenerationByConversationKey.get(key) ?? 0) + 1;
    saveGenerationByConversationKey.set(key, generation);
    return generation;
  }

  function persist<T>(operation: () => Promise<T>) {
    const draftOperation = persistenceTail.then(operation, operation);
    persistenceTail = draftOperation.catch(() => undefined);
    return draftOperation;
  }

  function read(spaceId: string, conversationId: string): ConversationComposerDraft | undefined {
    const draft = draftByConversationKey.value[`${spaceId}:${conversationId}`];
    return (
      draft && {
        prompt: draft.prompt,
        attachments: [...draft.attachments],
        extensions: cloneDeep(draft.extensions),
        reacquireFileNames: [...draft.reacquireFileNames],
      }
    );
  }

  function isStored(spaceId: string) {
    return spaces.spacePersistence(spaceId) === "persisted";
  }

  async function load(spaceId: string, conversationId: string) {
    if (!isStored(spaceId)) return read(spaceId, conversationId);
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
    draftByConversationKey.value = {
      ...draftByConversationKey.value,
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
    if (!snapshot || !isStored(spaceId)) return;
    const persisted = await persist(() =>
      saveConversationDraft({
        spaceId,
        conversationId,
        ...snapshot,
      }),
    );
    if (saveGenerationByConversationKey.get(key) !== generation) return;
    const current = read(spaceId, conversationId);
    if (current)
      saveCached(spaceId, conversationId, {
        ...current,
        reacquireFileNames: persisted.reacquireFileNames,
      });
  }

  async function move(spaceId: string, fromConversationId: string, toConversationId: string) {
    if (!draftByConversationKey.value[`${spaceId}:${fromConversationId}`]) return;
    advanceSaveGeneration(`${spaceId}:${fromConversationId}`);
    advanceSaveGeneration(`${spaceId}:${toConversationId}`);
    await persist(() => moveConversationDraft(spaceId, fromConversationId, toConversationId));
    const next = { ...draftByConversationKey.value };
    const draft = next[`${spaceId}:${fromConversationId}`];
    if (!draft) return;
    delete next[`${spaceId}:${fromConversationId}`];
    next[`${spaceId}:${toConversationId}`] = draft;
    draftByConversationKey.value = next;
  }

  async function remove(spaceId: string, conversationId: string) {
    advanceSaveGeneration(`${spaceId}:${conversationId}`);
    await persist(() => removeConversationDraft(spaceId, conversationId));
    const next = { ...draftByConversationKey.value };
    delete next[`${spaceId}:${conversationId}`];
    draftByConversationKey.value = next;
  }

  return { read, load, save, move, remove };
});
