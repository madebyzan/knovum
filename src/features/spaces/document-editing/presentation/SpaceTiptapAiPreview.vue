<script setup lang="ts">
import { X } from "@lucide/vue";
import { nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { EditorContextReference } from "../../../../shared/contracts/generated/EditorContextReference";
import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";

const props = defineProps<{
  showBody: boolean;
  streaming: boolean;
  errorMessage?: string | undefined;
  draftText: string;
  originalText?: string | undefined;
  showCompare?: boolean | undefined;
  thinking: boolean;
  attachedReferences: readonly EditorContextReference[];
}>();

const emit = defineEmits<{ removeReference: [nodeId: string] }>();
const { t } = useI18n();
const scrollBoxRef = useTemplateRef<HTMLElement>("scrollBoxRef");
let pinnedToBottom = true;

function handlePreviewScroll() {
  const box = scrollBoxRef.value;
  if (!box) return;
  pinnedToBottom = box.scrollHeight - box.scrollTop - box.clientHeight < 48;
}

watch(
  () => props.streaming,
  (active) => {
    if (active) pinnedToBottom = true;
  },
);

watch(
  () => props.draftText,
  async () => {
    if (!props.streaming || !pinnedToBottom) return;
    await nextTick();
    const box = scrollBoxRef.value;
    if (box) box.scrollTop = box.scrollHeight;
  },
);
</script>

<template>
  <div v-if="showBody" class="border-b border-stroke-divider bg-surface-muted/55">
    <div v-if="errorMessage" class="px-3.5 py-2.5 text-caption text-fg-danger">
      {{ errorMessage }}
    </div>
    <div v-else-if="draftText" class="px-3.5 py-3">
      <div
        ref="scrollBoxRef"
        class="max-h-[42vh] overflow-y-auto overscroll-contain outline-none"
        @scroll.passive="handlePreviewScroll"
      >
        <div
          v-if="showCompare && originalText"
          data-testid="tiptap-ai-compare"
          class="mb-2.5 rounded-md border border-stroke-divider bg-surface-canvas/70 px-2.5 py-2"
        >
          <p class="mb-1 text-caption-sm font-medium text-fg-tertiary">
            {{ t("spaces.richtext.ai.panel.original") }}
          </p>
          <p
            class="text-caption leading-relaxed whitespace-pre-wrap text-fg-tertiary line-through decoration-fg-danger/45"
          >
            {{ originalText }}
          </p>
        </div>
        <div v-if="showCompare && originalText" class="mb-1 text-caption-sm font-medium text-fg-secondary">
          {{ t("spaces.richtext.ai.panel.proposed") }}
        </div>
        <UiMarkdown :content="draftText" class="text-body-lg leading-relaxed text-fg-primary" />
      </div>
      <div v-if="streaming" class="mt-2 flex items-center gap-1.5 text-caption-sm text-fg-tertiary">
        <UiKnovumAiIcon class="size-3.5 animate-pulse text-fg-secondary" />
        <span>{{ t("spaces.richtext.ai.panel.generating") }}</span>
      </div>
    </div>
    <div v-else class="space-y-2.5 px-3.5 py-3">
      <div class="flex items-center gap-2 text-caption text-fg-secondary">
        <UiKnovumAiIcon class="size-3.5 animate-pulse text-fg-accent" />
        <span>{{
          thinking ? t("spaces.richtext.ai.panel.thinking") : t("spaces.richtext.ai.panel.generating")
        }}</span>
      </div>
      <div class="space-y-1.5" aria-hidden="true">
        <div class="h-2 w-4/5 animate-pulse rounded-full bg-surface-control-hover/85" />
        <div class="h-2 w-full animate-pulse rounded-full bg-surface-control-hover/65" />
        <div class="h-2 w-2/5 animate-pulse rounded-full bg-surface-control-hover/50" />
      </div>
    </div>
  </div>

  <div v-if="attachedReferences.length" class="flex flex-wrap items-center gap-1 px-3 pt-2">
    <span
      v-for="item in attachedReferences"
      :key="item.nodeId"
      class="inline-flex h-5.5 items-center gap-1 rounded-md bg-surface-control px-2 text-caption-sm text-fg-secondary"
    >
      <span class="max-w-40 truncate">{{ item.title }}</span>
      <button
        type="button"
        class="text-fg-tertiary transition-colors hover:text-fg-primary"
        :aria-label="t('common.remove')"
        @click="emit('removeReference', item.nodeId)"
      >
        <X class="size-3" :stroke-width="1.6" />
      </button>
    </span>
  </div>
</template>
