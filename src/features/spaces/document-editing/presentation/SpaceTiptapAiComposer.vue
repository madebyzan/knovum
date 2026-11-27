<script setup lang="ts">
import type { Editor } from "@tiptap/vue-3";
import { onClickOutside } from "@vueuse/core";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { isComposing } from "../../../../foundation/dom/composition";
import { POPOVER_SURFACE_CLASSES } from "../../../../shared/ui/ui-classes";
import type { SpaceAiReferenceCandidate } from "../tiptap-space-references";
import type { SpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";
import SpaceTiptapAiCommands from "./SpaceTiptapAiCommands.vue";
import SpaceTiptapAiPreview from "./SpaceTiptapAiPreview.vue";
import SpaceTiptapAiPromptBar from "./SpaceTiptapAiPromptBar.vue";
import SpaceTiptapAiReferenceSuggestions from "./SpaceTiptapAiReferenceSuggestions.vue";
import SpaceTiptapAiReviewActions from "./SpaceTiptapAiReviewActions.vue";
import { useSpaceTiptapAiMentions } from "./use-space-tiptap-ai-mentions";

const props = defineProps<{
  editor: Editor;
  writing: SpaceTiptapWritingTools;
  references: readonly SpaceAiReferenceCandidate[];
  fullWidth?: boolean;
}>();

const { t } = useI18n();
const composerRef = useTemplateRef<HTMLElement>("composerRef");
const promptBarRef = useTemplateRef<InstanceType<typeof SpaceTiptapAiPromptBar>>("promptBarRef");
const commandsRef = useTemplateRef<InstanceType<typeof SpaceTiptapAiCommands>>("commandsRef");
const highlightedIndex = shallowRef(0);
const showCompare = shallowRef(false);

const { phase, writingTarget, promptText, attachedReferences } = props.writing;
const open = computed(() => phase.value !== "closed");
const isStreaming = computed(() => phase.value === "streaming");
const isReview = computed(() => phase.value === "review");
const hasSelection = computed(() => Boolean(props.writing.activeSource.value?.hasSelection));
const canSubmit = computed(() => Boolean(promptText.value.trim()));
const attachedIds = computed(() => new Set(attachedReferences.value.map((item) => item.nodeId)));
const inputRef = computed(() => promptBarRef.value?.inputEl ?? null);

const mentions = useSpaceTiptapAiMentions({
  promptText,
  isStreaming,
  references: () => props.references,
  inputRef,
  attachReference: props.writing.attachReference,
});
const showMentionList = computed(() => Boolean(mentions.activeMention.value));
const showCommands = computed(() => !showMentionList.value && phase.value === "prompt" && !canSubmit.value);

onClickOutside(composerRef, (event) => {
  if (phase.value !== "prompt" || promptText.value.trim()) return;
  const target = event.target instanceof Element ? event.target : null;
  if (target?.closest("[data-reka-popper-content-wrapper], [role='listbox'], [role='menu']")) return;
  props.writing.discard();
});

watch(
  [() => phase.value, () => props.writing.activeSource.value],
  async ([nextPhase]) => {
    if (nextPhase !== "review") showCompare.value = false;
    if (nextPhase === "closed") return;
    highlightedIndex.value = 0;
    mentions.resetDismissed();
    await nextTick();
    if (nextPhase === "prompt" || nextPhase === "review") {
      inputRef.value?.focus();
      mentions.syncCaret();
    }
  },
  { flush: "post" },
);

function dismissAndRefocus() {
  props.writing.discard();
  props.editor.commands.focus();
}

function toggleReference(candidate: SpaceAiReferenceCandidate) {
  if (attachedIds.value.has(candidate.nodeId)) props.writing.removeReference(candidate.nodeId);
  else props.writing.attachReference(candidate);
}

function moveHighlightedCommand(delta: number) {
  const total = commandsRef.value?.count ?? 0;
  if (total) highlightedIndex.value = (highlightedIndex.value + delta + total) % total;
}

function handleEnterKey(event: KeyboardEvent) {
  event.preventDefault();
  if (canSubmit.value) return void props.writing.submitPrompt();
  if (showCommands.value && commandsRef.value?.executeHighlighted()) return;
  if (!isReview.value) return;
  if (event.shiftKey && hasSelection.value) props.writing.acceptInsertBelow();
  else if (!event.shiftKey) props.writing.acceptReplace();
}

function handleInputKeydown(event: KeyboardEvent) {
  if (isComposing(event) || mentions.handleMentionKeydown(event)) return;
  if (event.key === "Escape") {
    event.preventDefault();
    if (isStreaming.value) void props.writing.cancel();
    else dismissAndRefocus();
    return;
  }
  if (showCommands.value && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
    event.preventDefault();
    moveHighlightedCommand(event.key === "ArrowDown" ? 1 : -1);
    return;
  }
  if (event.key === "Enter") handleEnterKey(event);
}
</script>

<template>
  <div
    v-if="open"
    class="pointer-events-none absolute inset-x-0 bottom-5 z-30 flex justify-center px-6 sm:px-14"
  >
    <section
      ref="composerRef"
      data-testid="tiptap-ai-panel"
      :aria-label="t('spaces.richtext.ai.title')"
      :class="[
        POPOVER_SURFACE_CLASSES,
        'pointer-events-auto w-full overflow-hidden shadow-lg',
        fullWidth ? 'max-w-4xl' : 'max-w-[700px]',
      ]"
    >
      <SpaceTiptapAiPreview
        :show-body="isStreaming || isReview || Boolean(writing.errorMessage.value)"
        :streaming="isStreaming"
        :error-message="writing.errorMessage.value"
        :draft-text="writing.draftText.value"
        :original-text="hasSelection ? writing.activeSource.value?.text : undefined"
        :show-compare="showCompare"
        :thinking="writing.thinking.value"
        :thinking-text="writing.thinkingText.value"
        :thinking-duration-ms="writing.thinkingDurationMs.value"
        :attached-references="attachedReferences"
        @remove-reference="writing.removeReference"
      />
      <SpaceTiptapAiPromptBar
        ref="promptBarRef"
        v-model:prompt-text="promptText"
        v-model:writing-target="writingTarget"
        :is-streaming="isStreaming"
        :is-review="isReview"
        :can-submit="canSubmit"
        :references="references"
        :attached-references="attachedReferences"
        @sync-caret="mentions.syncCaret"
        @keydown="handleInputKeydown"
        @toggle-reference="toggleReference"
        @cancel="writing.cancel"
        @submit="writing.submitPrompt"
        @close="dismissAndRefocus"
      />
      <SpaceTiptapAiReferenceSuggestions
        v-if="showMentionList"
        :candidates="mentions.mentionCandidates.value"
        :attached-ids="attachedIds"
        :selected-index="mentions.mentionIndex.value"
        @highlight="(index) => (mentions.mentionIndex.value = index)"
        @select="mentions.pickMention"
      />
      <SpaceTiptapAiCommands
        v-else-if="showCommands"
        ref="commandsRef"
        :writing="writing"
        :has-selection="hasSelection"
        :selected-index="highlightedIndex"
        @highlight="(index) => (highlightedIndex = index)"
        @run-action="(action) => writing.run(action)"
        @run-translation="(target) => writing.runTranslation(target)"
      />
      <SpaceTiptapAiReviewActions
        v-if="isReview || phase === 'error'"
        :is-review="isReview"
        :has-selection="hasSelection"
        :show-compare="showCompare"
        @retry="writing.retry"
        @toggle-compare="showCompare = !showCompare"
        @discard="dismissAndRefocus"
        @insert-below="writing.acceptInsertBelow"
        @accept="writing.acceptReplace"
      />
    </section>
  </div>
</template>
