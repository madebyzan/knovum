import { computed, nextTick, shallowRef, watch, type ComputedRef, type Ref } from "vue";

import { findActiveMentionToken, stripMentionToken } from "../tiptap-ai-mention";
import { filterSpaceAiReferences, type SpaceAiReferenceCandidate } from "../tiptap-space-references";

interface AiMentionOptions {
  readonly promptText: Ref<string>;
  readonly isStreaming: ComputedRef<boolean>;
  readonly references: () => readonly SpaceAiReferenceCandidate[];
  readonly inputRef: Readonly<Ref<HTMLInputElement | null>>;
  readonly attachReference: (candidate: SpaceAiReferenceCandidate) => void;
}

export function useSpaceTiptapAiMentions(options: AiMentionOptions) {
  const mentionIndex = shallowRef(0);
  const caretPos = shallowRef(0);
  const dismissedAt = shallowRef<number | null>(null);

  const activeMention = computed(() => {
    if (options.isStreaming.value) return null;
    const token = findActiveMentionToken(options.promptText.value, caretPos.value);
    return !token || dismissedAt.value === token.start ? null : token;
  });

  const mentionCandidates = computed(() =>
    activeMention.value
      ? filterSpaceAiReferences(options.references(), activeMention.value.query).slice(0, 8)
      : [],
  );

  function syncCaret() {
    caretPos.value = options.inputRef.value?.selectionStart ?? options.promptText.value.length;
    if (dismissedAt.value !== null && !findActiveMentionToken(options.promptText.value, caretPos.value)) {
      dismissedAt.value = null;
    }
  }

  function resetDismissed() {
    dismissedAt.value = null;
  }

  function pickMention(candidate: SpaceAiReferenceCandidate) {
    const token = activeMention.value;
    options.attachReference(candidate);
    if (!token) return;
    const stripped = stripMentionToken(options.promptText.value, token);
    options.promptText.value = stripped.text;
    caretPos.value = stripped.caret;
    void nextTick(() => {
      options.inputRef.value?.focus();
      options.inputRef.value?.setSelectionRange(stripped.caret, stripped.caret);
    });
  }

  function handleMentionKeydown(event: KeyboardEvent): boolean {
    const token = activeMention.value;
    if (!token) return false;
    if (event.key === "Escape") {
      event.preventDefault();
      dismissedAt.value = token.start;
      return true;
    }
    const total = mentionCandidates.value.length;
    if (!total) return false;
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      mentionIndex.value = (mentionIndex.value + (event.key === "ArrowDown" ? 1 : -1) + total) % total;
      return true;
    }
    if (event.key === "Enter" || event.key === "Tab") {
      event.preventDefault();
      const chosen = mentionCandidates.value[mentionIndex.value];
      if (chosen) pickMention(chosen);
      return true;
    }
    return false;
  }

  watch(activeMention, () => {
    mentionIndex.value = 0;
  });

  return {
    activeMention,
    mentionCandidates,
    mentionIndex,
    syncCaret,
    resetDismissed,
    pickMention,
    handleMentionKeydown,
  };
}
