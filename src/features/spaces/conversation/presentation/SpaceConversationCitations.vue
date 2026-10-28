<script setup lang="ts">
import { ChevronRight } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import type { GroundedCitation } from "../../knowledge/model/schema/grounded-citation";

/** The sources an answer cites, each opening its passage while it is still current. */
defineProps<{ citations: readonly GroundedCitation[] }>();
const emit = defineEmits<{ open: [citation: GroundedCitation] }>();
const { t } = useI18n();
</script>

<template>
  <details class="group/sources w-full text-caption">
    <summary
      class="inline-flex cursor-pointer list-none items-center gap-1 rounded-md px-1 py-0.5 text-fg-secondary select-none hover:text-fg-primary [&::-webkit-details-marker]:hidden"
    >
      {{ t("spaces.conversation.sources", { count: citations.length }) }}
      <ChevronRight class="size-3.5 transition-transform group-open/sources:rotate-90" />
    </summary>
    <ol class="mt-1.5 divide-y divide-stroke-divider">
      <li v-for="citation in citations" :key="citation.index">
        <component
          :is="citation.state === 'current' ? 'button' : 'div'"
          :type="citation.state === 'current' ? 'button' : undefined"
          class="flex w-full min-w-0 gap-2 rounded-md px-1 py-2 text-left"
          :class="citation.state === 'current' ? 'hover:bg-surface-control' : ''"
          @click="emit('open', citation)"
        >
          <span
            class="mt-px inline-flex h-4 min-w-4 shrink-0 items-center justify-center rounded-sm bg-surface-control px-1 text-caption-sm text-fg-secondary tabular-nums"
            >{{ citation.index }}</span
          >
          <span class="min-w-0 flex-1">
            <span class="flex min-w-0 items-baseline gap-1.5">
              <span
                class="truncate font-medium"
                :class="citation.state === 'current' ? 'text-fg-primary' : 'text-fg-secondary'"
                >{{ citation.nodeTitle }}</span
              >
              <span v-if="citation.state === 'detached'" class="shrink-0 text-fg-tertiary">{{
                t("spaces.conversation.evidence.stale")
              }}</span>
            </span>
            <span class="mt-0.5 line-clamp-2 text-fg-secondary">{{ citation.quote }}</span>
          </span>
        </component>
      </li>
    </ol>
  </details>
</template>
