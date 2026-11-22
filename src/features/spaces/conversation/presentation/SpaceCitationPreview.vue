<script setup lang="ts">
import { ArrowUpRight } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { knowledgeLocationLabel } from "../../knowledge/model/location-label";
import type { GroundedCitation } from "../../knowledge/model/schema/grounded-citation";
import type { CitationPreviewState } from "./use-citation-hover";

const props = defineProps<{ state?: CitationPreviewState | undefined }>();
const emit = defineEmits<{
  open: [citation: GroundedCitation];
  previewEnter: [];
  previewLeave: [];
}>();
const { t } = useI18n();

const citation = computed(() => props.state?.citation);
const locationLabel = computed(() =>
  citation.value ? knowledgeLocationLabel(citation.value.locator) : undefined,
);
const isCurrent = computed(() => citation.value?.state === "current");

function handleOpen() {
  if (citation.value && isCurrent.value) emit("open", citation.value);
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-100 ease-out"
      enter-from-class="opacity-0"
      leave-active-class="transition-opacity duration-100 ease-in"
      leave-to-class="opacity-0"
    >
      <div
        v-if="state && citation"
        role="tooltip"
        data-citation-preview
        :style="state.style"
        class="fixed z-50 w-76 rounded-panel border border-stroke-strong material-overlay p-2.5 shadow-popover select-none"
        :class="isCurrent && 'cursor-pointer'"
        @pointerenter="emit('previewEnter')"
        @pointerleave="emit('previewLeave')"
        @click="handleOpen"
      >
        <div class="flex min-w-0 items-center gap-1.5">
          <span
            class="inline-flex h-4 min-w-4 shrink-0 items-center justify-center rounded-sm px-1 text-caption-sm font-medium tabular-nums"
            :class="
              isCurrent ? 'bg-surface-accent-subtle text-fg-accent' : 'bg-surface-control text-fg-secondary'
            "
          >
            {{ citation.index }}
          </span>
          <span class="min-w-0 flex-1 truncate text-caption font-medium text-fg-primary">
            {{ citation.nodeTitle }}
          </span>
          <span
            v-if="locationLabel"
            class="shrink-0 rounded-sm bg-surface-control px-1.5 py-0.5 font-mono text-caption-sm text-fg-secondary tabular-nums"
          >
            {{ locationLabel }}
          </span>
          <ArrowUpRight
            v-if="isCurrent"
            class="size-3.5 shrink-0 text-fg-tertiary"
            :stroke-width="1.75"
            aria-hidden="true"
          />
        </div>
        <p class="mt-1.5 line-clamp-4 text-caption leading-relaxed text-fg-secondary select-text">
          {{ citation.quote }}
        </p>
        <p v-if="!isCurrent" class="mt-1.5 text-caption-sm text-fg-tertiary">
          {{ t("spaces.conversation.evidence.stale") }}
        </p>
      </div>
    </Transition>
  </Teleport>
</template>
