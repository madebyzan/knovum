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

import UiButton from "../../../shared/ui/UiButton.vue";
import UiKbd from "../../../shared/ui/UiKbd.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import { highlightedSnippet } from "../search/highlighted-snippet";
import type { GlobalSearchMatch } from "../search/search-match";
import { useGlobalFind } from "../search/use-global-find";
import { knowledgeLocationLabel } from "./knowledge/knowledge-location-label";
import { nodeIcon } from "./node-icon";

const open = defineModel<boolean>("open", { required: true });
const { t } = useI18n();
const { query, matches, searching, failed, retry, select } = useGlobalFind(open);

function matchKey(match: GlobalSearchMatch) {
  return match.kind === "structure"
    ? `${match.spaceId}:structure:${match.structureNodeId}`
    : `${match.spaceId}:content:${match.chunkId}`;
}

function matchText(match: GlobalSearchMatch) {
  return match.kind === "structure"
    ? `${match.title} ${match.spaceTitle}`
    : `${match.nodeTitle} ${match.spaceTitle} ${match.text}`;
}
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-40 bg-surface-scrim/60" />
      <DialogContent
        :aria-describedby="undefined"
        class="fixed top-[12%] left-1/2 z-50 w-[calc(100vw-3rem)] max-w-156 -translate-x-1/2 overflow-hidden rounded-xl border border-stroke-default bg-surface-overlay shadow-dialog outline-none"
      >
        <DialogTitle class="sr-only">{{ t("spaces.search.allSpaces.title") }}</DialogTitle>
        <ListboxRoot>
          <div class="flex h-12 items-center gap-2.5 border-b border-stroke-divider px-3.5">
            <Search class="size-3.5 text-fg-tertiary" />
            <ListboxFilter
              v-model="query"
              autofocus
              maxlength="512"
              :placeholder="t('spaces.search.allSpaces.placeholder')"
              class="h-full min-w-0 flex-1 border-0 bg-transparent text-body-lg outline-none placeholder:text-fg-tertiary"
            />
            <UiSpinner v-if="searching" size="sm" class="text-fg-tertiary" />
            <UiKbd shortcut="esc" />
          </div>
          <ListboxContent class="max-h-90 min-h-48 space-y-0.5 overflow-y-auto p-1.5">
            <p v-if="!query.trim()" class="px-3 py-8 text-center text-caption text-fg-tertiary">
              {{ t("spaces.search.allSpaces.hint") }}
            </p>
            <ListboxItem
              v-for="match in matches"
              :key="matchKey(match)"
              :value="match"
              :text-value="matchText(match)"
              @select="select(match)"
              class="flex min-h-(--knovum-command-palette-item-height) w-full cursor-default items-start gap-2.5 rounded-md px-2.5 py-2.5 text-left outline-none data-highlighted:bg-(--knovum-sidebar-item-bg-selected)"
            >
              <span
                class="mt-0.5 grid size-7 shrink-0 place-items-center rounded-md bg-surface-control text-fg-secondary"
              >
                <LayoutPanelTop
                  v-if="match.kind === 'structure' && match.nodeKind === 'section'"
                  class="size-3.5"
                />
                <Group v-else-if="match.kind === 'structure'" class="size-3.5" />
                <component v-else :is="nodeIcon(match.nodeKind)" class="size-3.5" />
              </span>
              <span class="min-w-0 flex-1">
                <span class="flex min-w-0 items-center gap-2">
                  <strong class="truncate text-label font-medium">{{
                    match.kind === "structure" ? match.title : match.nodeTitle
                  }}</strong>
                  <span v-if="match.kind === 'content'" class="shrink-0 text-caption-sm text-fg-tertiary">
                    {{ knowledgeLocationLabel(match.locator) }}
                  </span>
                </span>
                <span class="mt-0.5 block truncate text-caption-sm text-fg-tertiary">{{
                  match.spaceTitle
                }}</span>
                <span v-if="match.kind === 'structure'" class="mt-0.5 block text-caption text-fg-secondary">
                  {{
                    match.nodeKind === "section"
                      ? t("spaces.canvas.kinds.section")
                      : t("spaces.canvas.kinds.group")
                  }}
                </span>
                <span v-else class="mt-0.5 line-clamp-2 block text-caption leading-relaxed text-fg-secondary">
                  <template v-for="(part, index) in highlightedSnippet(match.snippet)" :key="index">
                    <mark v-if="part.match" class="rounded-sm bg-surface-selected px-0.5 text-fg-primary">{{
                      part.text
                    }}</mark>
                    <template v-else>{{ part.text }}</template>
                  </template>
                </span>
              </span>
            </ListboxItem>
            <div v-if="query.trim() && failed" class="grid justify-items-center gap-2 px-3 py-8">
              <p class="text-caption text-fg-danger">{{ t("spaces.search.failed") }}</p>
              <UiButton size="sm" variant="secondary" @click="retry()">
                {{ t("common.retry") }}
              </UiButton>
            </div>
            <p
              v-else-if="query.trim() && !searching && matches.length === 0"
              class="px-3 py-8 text-center text-caption text-fg-tertiary"
            >
              {{ t("spaces.search.allSpaces.noResults") }}
            </p>
          </ListboxContent>
        </ListboxRoot>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
