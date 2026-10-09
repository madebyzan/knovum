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

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiKbd from "../../../../shared/ui/UiKbd.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import { useFind } from "../../search/use-find";
import { nodeIcon } from "../canvas/node-icon";
import SpaceFindKnowledgeMatch from "./SpaceFindKnowledgeMatch.vue";

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
} = useFind(open);
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-40 bg-surface-scrim/60" />
      <DialogContent
        data-app-command-scope=""
        class="fixed top-[14%] left-1/2 z-50 w-[calc(100vw-3rem)] max-w-156 -translate-x-1/2 overflow-hidden rounded-xl border border-stroke-default bg-surface-overlay shadow-dialog outline-none"
      >
        <DialogTitle class="sr-only">{{ t("spaces.search.find.title") }}</DialogTitle>
        <ListboxRoot @update:model-value="select">
          <div class="flex h-12 items-center gap-2.5 border-b border-stroke-divider px-3.5">
            <Search class="size-3.5 text-fg-tertiary" />
            <ListboxFilter
              v-model="query"
              autofocus
              :placeholder="t('spaces.search.find.placeholder')"
              class="h-full min-w-0 flex-1 border-0 bg-transparent text-body-lg outline-none placeholder:text-fg-tertiary"
            />
            <UiSpinner v-if="searching" class="text-fg-tertiary" />
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
          <ListboxContent class="max-h-90 min-h-48 space-y-0.5 overflow-y-auto p-1.5">
            <p
              v-if="!query.trim() && defaultNodes.length"
              class="px-2.5 py-1 text-caption-sm font-medium tracking-label-uppercase text-fg-tertiary uppercase"
            >
              {{ t("spaces.search.find.defaultTitle") }}
            </p>
            <ListboxItem
              v-for="node in !query.trim() ? defaultNodes : []"
              :key="node.id"
              :value="`node:${node.id}`"
              :text-value="node.data.title"
              class="flex min-h-(--knovum-command-palette-item-height) w-full cursor-default items-center gap-2.5 rounded-md px-2.5 py-2 text-left outline-none data-highlighted:bg-(--knovum-sidebar-item-bg-selected)"
            >
              <span
                class="grid size-7 shrink-0 place-items-center rounded-md bg-surface-control text-fg-secondary"
                ><component :is="nodeIcon(node.data.kind)" class="size-3.5"
              /></span>
              <span class="min-w-0 flex-1"
                ><strong class="block truncate text-label font-medium">{{ node.data.title }}</strong
                ><span class="block truncate text-caption text-fg-tertiary">{{
                  ("fileName" in node.data && node.data.fileName) ||
                  ("siteName" in node.data && node.data.siteName) ||
                  t("spaces.search.find.defaultContentLabel")
                }}</span></span
              >
            </ListboxItem>
            <ListboxItem
              v-for="match in query.trim() ? structureMatches : []"
              :key="match.structureNodeId"
              :value="`structure:${match.structureNodeId}`"
              :text-value="match.title"
              class="flex min-h-(--knovum-command-palette-item-height) w-full cursor-default items-center gap-2.5 rounded-md px-2.5 py-2 text-left outline-none data-highlighted:bg-(--knovum-sidebar-item-bg-selected)"
            >
              <span
                class="grid size-7 shrink-0 place-items-center rounded-md bg-surface-control text-fg-secondary"
              >
                <LayoutPanelTop v-if="match.nodeKind === 'section'" class="size-3.5" />
                <Group v-else class="size-3.5" />
              </span>
              <span class="min-w-0 flex-1">
                <strong class="block truncate text-label font-medium">{{ match.title }}</strong>
                <span class="block text-caption text-fg-tertiary">{{
                  match.nodeKind === "section"
                    ? t("spaces.canvas.kinds.section")
                    : t("spaces.canvas.kinds.group")
                }}</span>
              </span>
            </ListboxItem>
            <SpaceFindKnowledgeMatch
              v-for="match in query.trim() ? matches : []"
              :key="match.chunkId"
              :match="match"
            />
            <p v-if="semanticSearch.preparing.value" class="px-2.5 py-1 text-caption-sm text-fg-tertiary">
              {{
                t(
                  semanticSearch.indexing.value
                    ? "spaces.search.semantic.indexing"
                    : "spaces.search.semantic.preparing",
                )
              }}
            </p>
            <div v-if="query.trim() && searchFailed" class="grid justify-items-center gap-2 px-3 py-8">
              <p class="text-caption text-fg-danger">{{ t("spaces.search.failed") }}</p>
              <UiButton size="sm" variant="secondary" @click="retry">{{ t("common.retry") }}</UiButton>
            </div>
            <p
              v-else-if="query.trim() && !searching && matches.length === 0 && structureMatches.length === 0"
              class="px-3 py-8 text-center text-caption text-fg-tertiary"
            >
              {{ t("spaces.search.find.noResults") }}
            </p>
            <p
              v-else-if="!query.trim() && defaultNodes.length === 0"
              class="px-3 py-8 text-center text-caption text-fg-tertiary"
            >
              {{ t("spaces.search.find.emptyHint") }}
            </p>
          </ListboxContent>
        </ListboxRoot>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
