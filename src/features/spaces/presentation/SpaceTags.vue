<script setup lang="ts">
import { Tag, X } from "@lucide/vue";
import { union, uniq, without } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import { useCanvasEditorSession, useCanvasSelectionProjection } from "../canvas/editor-session";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { createTagChanges } from "../model/nodes/tag-changes";
import { createStructureIndex } from "../model/structure/structure-index";
import { sectionAncestorOf } from "../model/structure/structure-queries";
import { useSpaceStore } from "../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem } from "../workspace/space-menu";
import { selectedTagCounts, spaceTagCounts, suggestTags } from "./tag-projections";

withDefaults(defineProps<{ mode?: "both" | "browse" | "selection" }>(), { mode: "both" });
const store = useSpaceStore();
const { nodes } = storeToRefs(store);
const editorSession = useCanvasEditorSession();
const nodeOperations = useNodeOperationGate();
const { selectedNodes } = useCanvasSelectionProjection(nodes);
const { t } = useI18n();
const draft = shallowRef("");
const selectedContentNodes = computed(() => selectedNodes.value);
const selectedIds = computed(() => selectedContentNodes.value.map((node) => node.id));
const selectionTagCounts = computed(() => selectedTagCounts(selectedContentNodes.value));
const tagCounts = computed(() => spaceTagCounts(nodes.value));
const suggestions = computed(() => suggestTags(draft.value, tagCounts.value, selectionTagCounts.value));

async function applyTag(tag = draft.value) {
  const value = tag.trim();
  const ids = [...selectedIds.value];
  const spaceId = store.activeSpaceId;
  const initialDraft = draft.value;
  if (!value || ids.length === 0 || !spaceId) return;
  await updateTags((tags) => union(tags, [value]));
  if (store.activeSpaceId === spaceId && draft.value === initialDraft) draft.value = "";
}

async function removeTag(tag: string) {
  await updateTags((tags) => without(tags, tag));
}

async function updateTags(update: (tags: readonly string[]) => string[]) {
  if (!nodeOperations.allows("edit")) return;
  const changes = createTagChanges(selectedContentNodes.value, update);
  if (changes.length) await store.updateNodeContents(changes);
}
async function revealTag(tag: string) {
  if (!nodeOperations.allows("structure")) return;
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const matches = nodes.value.filter((node) => node.data.tags?.includes(tag));
  const index = createStructureIndex(store.editSnapshot.structure);
  const sectionIds = uniq(
    matches.flatMap((node) => {
      const structureNodeId = index.nodeIdToStructureNodeId.get(node.id);
      const section = structureNodeId ? sectionAncestorOf(structureNodeId, index) : undefined;
      return section ? [section.id] : [];
    }),
  );
  for (const id of sectionIds) {
    if (store.activeSpaceId !== spaceId) return;
    const collapsed = store.editSnapshot.layouts.some(
      (layout) => layout.structureNodeId === id && layout.kind === "section" && layout.collapsed,
    );
    if (!collapsed) continue;
    // oxlint-disable-next-line eslint/no-await-in-loop -- Space edits are revision-guarded and must not overlap.
    await store.executeSpaceEdit({ kind: "section.set-collapsed", structureNodeId: id, collapsed: false });
  }
  if (store.activeSpaceId !== spaceId) return;
  const ids = matches.map((node) => node.id);
  editorSession.reveal(ids, "focus");
}
async function showTagMenu(tag: string) {
  await popupSpaceContextMenu([[spaceMenuItem(t("spaces.tags.showNodes"), () => revealTag(tag))]]);
}
</script>

<template>
  <section v-if="mode !== 'selection'">
    <h3 class="px-1 text-caption font-semibold text-fg-secondary">
      {{ t("spaces.tags.title") }}
    </h3>
    <div v-if="tagCounts.length" class="mt-2 space-y-0.5">
      <button
        v-for="[tag, count] in tagCounts"
        :key="tag"
        type="button"
        :disabled="!nodeOperations.allows('structure')"
        class="flex h-(--knovum-space-tag-row-height) w-full items-center gap-2 rounded-md px-2 text-left text-caption text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary"
        @click="revealTag(tag)"
        @contextmenu.prevent="showTagMenu(tag)"
      >
        <Tag class="size-3 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
        <span class="min-w-0 flex-1 truncate">{{ tag }}</span>
        <UiBadge size="sm">{{ count }}</UiBadge>
      </button>
    </div>
    <p v-else class="mt-2 px-1 text-caption text-fg-tertiary">
      {{ t("spaces.tags.empty") }}
    </p>
  </section>

  <section v-if="mode !== 'browse'">
    <div v-if="selectedIds.length === 0" class="px-1 text-caption text-fg-tertiary">
      {{ t("spaces.tags.selectionHint") }}
    </div>
    <template v-else>
      <div class="relative">
        <UiInput
          v-model="draft"
          :disabled="!nodeOperations.allows('edit')"
          class="w-full"
          :placeholder="t('spaces.tags.add')"
          @keydown.enter="applyTag()"
        />
        <div
          v-if="suggestions.length"
          class="absolute top-full left-0 z-20 mt-1 w-full rounded-panel bg-surface-overlay p-1 shadow-popover ring-1 ring-stroke-default"
        >
          <button
            v-for="[tag, count] in suggestions"
            :key="tag"
            type="button"
            :disabled="!nodeOperations.allows('edit')"
            class="flex h-(--knovum-menu-item-height) w-full items-center justify-between rounded-md px-2 text-left text-caption hover:bg-surface-control-hover"
            @click="applyTag(tag)"
          >
            <span class="truncate">{{ tag }}</span>
            <span class="text-caption-sm text-fg-tertiary">{{ count }}</span>
          </button>
        </div>
      </div>
      <div v-if="selectionTagCounts.size" class="mt-2 flex flex-wrap gap-1.5">
        <span
          v-for="[tag, count] in selectionTagCounts"
          :key="tag"
          class="inline-flex h-(--knovum-control-height-sm) items-center gap-1 rounded-md bg-surface-control pr-1 pl-2 text-caption text-fg-secondary"
        >
          {{ tag }}
          <span v-if="count !== selectedIds.length" class="text-caption-sm text-fg-tertiary"
            >{{ count }}/{{ selectedIds.length }}</span
          >
          <button
            type="button"
            :disabled="!nodeOperations.allows('edit')"
            class="grid size-5 place-items-center rounded-md hover:bg-surface-control-hover"
            :aria-label="t('spaces.tags.remove', { tag })"
            @click="removeTag(tag)"
          >
            <X class="size-3" />
          </button>
        </span>
      </div>
    </template>
  </section>
</template>
