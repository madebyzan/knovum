<script setup lang="ts">
import { Group, LayoutPanelTop, Search } from "@lucide/vue";
import {
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  ListboxContent,
  ListboxFilter,
  ListboxItem,
  ListboxRoot,
} from "reka-ui";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiKbd from "../../../shared/ui/UiKbd.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import { matchLocatorFor, type SpaceMatchLocator } from "../knowledge/model/match-locator";
import type { KnowledgeSearchMatch } from "../knowledge/model/schema/search-match";
import { knowledgeLocationLabel } from "../knowledge/presentation/location-label";
import { formatDurationMs } from "../media/format-duration";
import { highlightedSnippet } from "../search/highlighted-snippet";
import { useFind } from "../search/use-find";
import { nodeIcon } from "./node-icon";

const open = defineModel<boolean>("open", { required: true });
const { t } = useI18n();
const {
  query,
  matches,
  structureMatches,
  searching,
  searchFailed,
  semanticSearch,
  defaultNodes,
  select,
  retry,
} = useFind(open, () => {
  open.value = false;
});

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
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-40 bg-scrim/60" />
      <DialogContent
        class="fixed top-[14%] left-1/2 z-50 w-[calc(100vw-3rem)] max-w-(--knovum-command-max-width) -translate-x-1/2 overflow-hidden rounded-dialog border border-border bg-overlay shadow-dialog outline-none"
      >
        <DialogTitle class="sr-only">{{ t("spaces.search.find.title") }}</DialogTitle>
        <ListboxRoot @update:model-value="select">
          <div
            class="flex h-(--knovum-command-input-height) items-center gap-2.5 border-b border-divider px-3.5"
          >
            <Search class="size-(--knovum-icon-size-sm) text-foreground-tertiary" />
            <ListboxFilter
              v-model="query"
              autofocus
              :placeholder="t('spaces.search.find.placeholder')"
              class="h-full min-w-0 flex-1 border-0 bg-transparent text-body-lg outline-none placeholder:text-foreground-tertiary"
            />
            <UiSpinner v-if="searching" class="text-foreground-tertiary" />
            <UiButton
              size="sm"
              variant="ghost"
              :disabled="semanticSearch.preparing.value || semanticSearch.enabled.value"
              :aria-pressed="semanticSearch.enabled.value"
              @click="semanticSearch.enable"
              >{{ t("spaces.search.semantic.enable") }}</UiButton
            >
            <UiKbd shortcut="esc" />
          </div>
          <ListboxContent
            class="max-h-(--knovum-command-list-max-height) min-h-48 space-y-0.5 overflow-y-auto p-1.5"
          >
            <p
              v-if="!query.trim() && defaultNodes.length"
              class="px-2.5 py-1 text-micro font-medium tracking-uppercase text-foreground-tertiary uppercase"
            >
              {{ t("spaces.search.find.defaultTitle") }}
            </p>
            <ListboxItem
              v-for="node in !query.trim() ? defaultNodes : []"
              :key="node.id"
              :value="node"
              :text-value="node.data.title"
              class="flex min-h-(--knovum-command-item-height) w-full cursor-default items-center gap-2.5 rounded-control px-2.5 py-2 text-left outline-none data-highlighted:bg-(--knovum-sidebar-item-background-selected)"
            >
              <span
                class="grid size-7 shrink-0 place-items-center rounded-control bg-control text-foreground-secondary"
                ><component :is="nodeIcon(node.data.kind)" class="size-(--knovum-icon-size-sm)"
              /></span>
              <span class="min-w-0 flex-1"
                ><strong class="block truncate text-label font-medium">{{ node.data.title }}</strong
                ><span class="block truncate text-caption text-foreground-tertiary">{{
                  ("fileName" in node.data && node.data.fileName) ||
                  ("siteName" in node.data && node.data.siteName) ||
                  t("spaces.search.find.defaultDetail")
                }}</span></span
              >
            </ListboxItem>
            <ListboxItem
              v-for="match in query.trim() ? structureMatches : []"
              :key="match.structureNodeId"
              :value="match"
              :text-value="match.title"
              class="flex min-h-(--knovum-command-item-height) w-full cursor-default items-center gap-2.5 rounded-control px-2.5 py-2 text-left outline-none data-highlighted:bg-(--knovum-sidebar-item-background-selected)"
            >
              <span
                class="grid size-7 shrink-0 place-items-center rounded-control bg-control text-foreground-secondary"
              >
                <LayoutPanelTop v-if="match.nodeKind === 'section'" class="size-(--knovum-icon-size-sm)" />
                <Group v-else class="size-(--knovum-icon-size-sm)" />
              </span>
              <span class="min-w-0 flex-1">
                <strong class="block truncate text-label font-medium">{{ match.title }}</strong>
                <span class="block text-caption text-foreground-tertiary">{{
                  match.nodeKind === "section"
                    ? t("spaces.canvas.kinds.section")
                    : t("spaces.canvas.kinds.group")
                }}</span>
              </span>
            </ListboxItem>
            <ListboxItem
              v-for="match in query.trim() ? matches : []"
              :key="match.chunkId"
              :value="match"
              :text-value="`${match.nodeTitle} ${match.text}`"
              class="flex min-h-(--knovum-command-item-height) w-full cursor-default items-start gap-2.5 rounded-control px-2.5 py-2.5 text-left outline-none data-highlighted:bg-(--knovum-sidebar-item-background-selected)"
            >
              <span
                class="mt-0.5 grid size-7 shrink-0 place-items-center rounded-control bg-control text-foreground-secondary"
                ><component :is="nodeIcon(match.nodeKind)" class="size-(--knovum-icon-size-sm)"
              /></span>
              <span class="min-w-0 flex-1">
                <span class="flex min-w-0 items-center gap-2"
                  ><strong class="truncate text-label font-medium">{{ match.nodeTitle }}</strong
                  ><UiBadge v-if="matchBadgeLabel(match)" size="sm"> {{ matchBadgeLabel(match) }} </UiBadge
                  ><span class="shrink-0 text-micro text-foreground-tertiary">{{
                    knowledgeLocationLabel(match.locator)
                  }}</span></span
                >
                <span class="mt-0.5 line-clamp-2 block text-caption leading-relaxed text-foreground-secondary"
                  ><template v-for="(part, index) in highlightedSnippet(match.snippet)" :key="index"
                    ><mark v-if="part.match" class="rounded-sm bg-selected px-0.5 text-foreground">{{
                      part.text
                    }}</mark
                    ><template v-else>{{ part.text }}</template></template
                  ></span
                >
              </span>
            </ListboxItem>
            <p v-if="semanticSearch.preparing.value" class="px-2.5 py-1 text-micro text-foreground-tertiary">
              {{
                t(
                  semanticSearch.indexing.value
                    ? "spaces.search.semantic.indexing"
                    : "spaces.search.semantic.preparing",
                )
              }}
            </p>
            <div v-if="query.trim() && searchFailed" class="grid justify-items-center gap-2 px-3 py-8">
              <p class="text-caption text-danger">{{ t("spaces.search.failed") }}</p>
              <UiButton size="sm" variant="secondary" @click="retry">{{ t("common.retry") }}</UiButton>
            </div>
            <p
              v-else-if="query.trim() && !searching && matches.length === 0 && structureMatches.length === 0"
              class="px-3 py-8 text-center text-caption text-foreground-tertiary"
            >
              {{ t("spaces.search.find.noResults") }}
            </p>
            <p
              v-else-if="!query.trim() && defaultNodes.length === 0"
              class="px-3 py-8 text-center text-caption text-foreground-tertiary"
            >
              {{ t("spaces.search.find.emptyHint") }}
            </p>
          </ListboxContent>
        </ListboxRoot>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
