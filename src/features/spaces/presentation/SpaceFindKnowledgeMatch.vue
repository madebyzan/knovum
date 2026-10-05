<script setup lang="ts">
import { ListboxItem } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../shared/ui/UiBadge.vue";
import { matchLocatorFor, type SpaceMatchLocator } from "../knowledge/model/match-locator";
import type { KnowledgeSearchMatch } from "../knowledge/model/schema/search-match";
import { formatDurationMs } from "../media/format-duration";
import { highlightedSnippet } from "../search/highlighted-snippet";
import { knowledgeLocationLabel } from "./knowledge/knowledge-location-label";
import { nodeIcon } from "./node-icon";

const props = defineProps<{ match: KnowledgeSearchMatch }>();
const { t } = useI18n();
const badgeLabel = computed(() => matchBadgeLabel(props.match));

function matchBadgeLabel(match: KnowledgeSearchMatch): string | null {
  if (match.metadata?.role === "document_ocr") return t("spaces.processing.matchBadge.documentOcr");
  if (match.metadata?.role === "document_interpretation")
    return t("spaces.processing.matchBadge.documentInterpretation");
  const locator: SpaceMatchLocator = matchLocatorFor(match);
  if (locator.kind === "image-description") return t("spaces.processing.matchBadge.imageDescription");
  if (locator.kind === "video-cover") return t("spaces.processing.matchBadge.videoCover");
  if (locator.kind === "media-time-range")
    return t("spaces.processing.matchBadge.mediaTimeRange", {
      start: formatDurationMs(locator.startMs),
      end: formatDurationMs(locator.endMs),
    });
  return null;
}
</script>

<template>
  <ListboxItem
    :value="`match:${match.chunkId}`"
    :text-value="`${match.nodeTitle} ${match.text}`"
    class="flex min-h-(--knovum-command-palette-item-height) w-full cursor-default items-start gap-2.5 rounded-md px-2.5 py-2.5 text-left outline-none data-highlighted:bg-(--knovum-sidebar-item-bg-selected)"
  >
    <span
      class="mt-0.5 grid size-7 shrink-0 place-items-center rounded-md bg-surface-control text-fg-secondary"
      ><component :is="nodeIcon(match.nodeKind)" class="size-3.5"
    /></span>
    <span class="min-w-0 flex-1">
      <span class="flex min-w-0 items-center gap-2"
        ><strong class="truncate text-label font-medium">{{ match.nodeTitle }}</strong
        ><UiBadge v-if="badgeLabel" size="sm"> {{ badgeLabel }} </UiBadge
        ><span class="shrink-0 text-caption-sm text-fg-tertiary">{{
          knowledgeLocationLabel(match.locator)
        }}</span></span
      >
      <span class="mt-0.5 line-clamp-2 block text-caption leading-relaxed text-fg-secondary"
        ><template v-for="(part, index) in highlightedSnippet(match.snippet)" :key="index"
          ><mark v-if="part.match" class="rounded-sm bg-surface-selected px-0.5 text-fg-primary">{{
            part.text
          }}</mark
          ><template v-else>{{ part.text }}</template></template
        ></span
      >
    </span>
  </ListboxItem>
</template>
