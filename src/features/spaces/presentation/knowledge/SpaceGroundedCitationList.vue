<script setup lang="ts">
import { ChevronDown, MapPin } from "@lucide/vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { useContextDock } from "../../canvas/use-context-dock";
import { groundedCitationCurrent } from "../../knowledge/assistant/grounded-citation-actions";
import { type CitationGroup, useCitationGroups } from "../../knowledge/assistant/use-citation-groups";
import { findGroundedCitation, type GroundedCitation } from "../../knowledge/model/grounded-citation";
import { knowledgeLocationLabel } from "../../knowledge/model/location";
import { useSpaceStore } from "../../session/space-store";

const props = defineProps<{ citations: readonly GroundedCitation[]; activeIndex?: number }>();
const emit = defineEmits<{ highlightCitation: [index: number] }>();
const store = useSpaceStore();
const dock = useContextDock();
const { t } = useI18n();
const { groups, iconFor } = useCitationGroups(() => props.citations);
const open = shallowRef(false);
const activeCitation = computed(() =>
  props.activeIndex ? findGroundedCitation(props.citations, props.activeIndex) : undefined,
);

watch(
  () => props.activeIndex,
  (index) => {
    if (index) open.value = true;
  },
);

function activate(citation: GroundedCitation, index: number) {
  open.value = true;
  emit("highlightCitation", citation.index ?? index);
}

function firstCitation(group: CitationGroup) {
  return group.citations[0]!;
}

function locate(citation: GroundedCitation) {
  if (citation.state !== "current" || !groundedCitationCurrent(store, citation)) return;
  dock.openSource(
    {
      nodeId: citation.nodeId,
      nodeTitle: citation.nodeTitle,
      chunkId: citation.chunkId,
      locator: citation.locator,
      text: citation.quote,
    },
    { reveal: true },
  );
}
</script>

<template>
  <CollapsibleRoot v-model:open="open" class="mt-3 border-t border-divider">
    <CollapsibleTrigger
      class="flex h-(--knovum-control-height-sm) w-full items-center gap-1 px-1 text-caption font-medium text-foreground-secondary outline-none hover:text-foreground"
    >
      <ChevronDown
        class="size-(--knovum-icon-size-xs) transition-transform data-[state=closed]:-rotate-90"
        :stroke-width="1.55"
      />
      {{ t("spaces.assistant.sources", { count: groups.length }) }}
    </CollapsibleTrigger>
    <CollapsibleContent>
      <div v-for="group in groups" :key="group.key" class="border-t border-divider first:border-t-0">
        <button
          type="button"
          class="flex w-full min-w-0 items-start gap-2 px-1 py-2 text-left outline-none hover:bg-control-hover"
          :class="group.citations.some(({ index }) => index === activeIndex) && 'bg-accent-subtle'"
          @click="activate(firstCitation(group).citation, firstCitation(group).index)"
        >
          <component
            :is="iconFor(group)"
            class="mt-0.5 size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
            :stroke-width="1.55"
          />
          <span class="min-w-0 flex-1">
            <span class="flex min-w-0 items-center gap-1.5">
              <span class="truncate text-caption font-medium text-foreground">{{ group.title }}</span>
              <span class="ml-auto shrink-0 text-micro text-foreground-tertiary">
                {{ group.citations.map(({ index }) => index).join(", ") }}
              </span>
            </span>
            <span class="mt-0.5 block truncate text-micro text-foreground-tertiary">
              {{ knowledgeLocationLabel(firstCitation(group).citation.locator) }}
            </span>
          </span>
        </button>
        <div
          v-if="activeCitation && group.citations.some(({ index }) => index === activeIndex)"
          class="border-t border-divider bg-accent-subtle px-3 py-2.5"
        >
          <p class="line-clamp-4 text-caption leading-relaxed text-foreground-secondary">
            “{{ activeCitation.quote }}”
          </p>
          <UiButton
            size="sm"
            variant="ghost"
            class="mt-1.5 -ml-2"
            :disabled="!groundedCitationCurrent(store, activeCitation)"
            @click="locate(activeCitation)"
          >
            <MapPin class="size-(--knovum-icon-size-xs)" />{{ t("spaces.assistant.showOnCanvas") }}
          </UiButton>
        </div>
      </div>
    </CollapsibleContent>
  </CollapsibleRoot>
</template>
