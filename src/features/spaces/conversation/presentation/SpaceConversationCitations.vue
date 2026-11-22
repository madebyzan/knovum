<script setup lang="ts">
import { ChevronRight } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import type { GroundedCitation } from "../../knowledge/model/schema/grounded-citation";
import { knowledgeLocationLabel } from "../../knowledge/presentation/knowledge-location-label";

/** The sources an answer cites, each opening its passage while it is still current. */
defineProps<{ citations: readonly GroundedCitation[] }>();
const emit = defineEmits<{ open: [citation: GroundedCitation] }>();
const { t } = useI18n();
</script>

<template>
  <details class="group/sources w-full text-caption">
    <summary
      class="-ml-1.5 inline-flex list-none items-center gap-1 rounded-md px-1.5 py-1 text-fg-secondary transition-colors select-none hover:bg-surface-control/70 hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus [&::-webkit-details-marker]:hidden"
    >
      {{ t("spaces.conversation.sources", { count: citations.length }) }}
      <ChevronRight
        class="size-3.5 text-fg-tertiary transition-transform duration-150 group-open/sources:rotate-90"
      />
    </summary>
    <ol class="mt-1 space-y-0.5 border-l border-stroke-divider/70 pl-2.5">
      <li v-for="citation in citations" :key="citation.index">
        <component
          :is="citation.state === 'current' ? 'button' : 'div'"
          :type="citation.state === 'current' ? 'button' : undefined"
          class="-mx-1.5 flex w-[calc(100%+0.75rem)] min-w-0 gap-2 rounded-md px-1.5 py-1.5 text-left transition-colors duration-100"
          :class="
            citation.state === 'current'
              ? 'hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus'
              : 'opacity-70'
          "
          @click="emit('open', citation)"
        >
          <span
            class="mt-px inline-flex h-4 min-w-4 shrink-0 items-center justify-center rounded-sm bg-surface-control px-1 text-caption-sm font-medium text-fg-secondary tabular-nums"
            >{{ citation.index }}</span
          >
          <span class="min-w-0 flex-1">
            <span class="flex min-w-0 items-baseline gap-1.5">
              <span
                class="min-w-0 flex-1 truncate font-medium"
                :class="citation.state === 'current' ? 'text-fg-primary' : 'text-fg-secondary'"
                >{{ citation.nodeTitle }}</span
              >
              <span
                v-if="citation.state === 'current'"
                class="shrink-0 font-mono text-caption-sm text-fg-tertiary tabular-nums"
                >{{ knowledgeLocationLabel(citation.locator) }}</span
              >
              <span v-else class="shrink-0 text-caption-sm text-fg-tertiary">{{
                t("spaces.conversation.evidence.stale")
              }}</span>
            </span>
            <span class="mt-0.5 line-clamp-2 leading-relaxed text-fg-secondary">{{ citation.quote }}</span>
          </span>
        </component>
      </li>
    </ol>
  </details>
</template>
