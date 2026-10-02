<script setup lang="ts">
import { Plus, Tag, X } from "@lucide/vue";
import { union, uniq, without } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import { getCanvasEditorSession, useCanvasSelectionProjection } from "../canvas/editor-session";
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
const editorSession = getCanvasEditorSession();
const nodeOperations = useNodeOperationGate();
const { selectedNodes } = useCanvasSelectionProjection(nodes);
const { t } = useI18n();
const draft = shallowRef("");
const saving = shallowRef(false);
const selectionTagCounts = computed(() => selectedTagCounts(selectedNodes.value));
const tagCounts = computed(() => spaceTagCounts(nodes.value));
const suggestions = computed(() => suggestTags(draft.value, tagCounts.value, selectionTagCounts.value));
async function applyTag(tag = draft.value) {
  const value = tag.trim();
  const spaceId = store.activeSpaceId;
  const initialDraft = draft.value;
  if (!value || selectedNodes.value.length === 0 || !spaceId) return;
  const saved = await updateTags((tags) => union(tags, [value]));
  if (saved && store.activeSpaceId === spaceId && draft.value === initialDraft) draft.value = "";
}

const removeTag = (tag: string) => updateTags((tags) => without(tags, tag));

async function updateTags(update: (tags: readonly string[]) => string[]) {
  if (saving.value || !nodeOperations.allows("edit")) return false;
  const changes = createTagChanges(selectedNodes.value, update);
  if (!changes.length) return true;
  saving.value = true;
  try {
    await store.updateNodeContents(changes);
    return true;
  } catch (error) {
    toast.error(getErrorMessage(error));
    return false;
  } finally {
    saving.value = false;
  }
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
        class="flex h-interaction w-full items-center gap-2 rounded-md px-2 text-left text-caption text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary disabled:opacity-40"
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
    <div v-if="selectedNodes.length === 0" class="px-1 text-caption text-fg-tertiary">
      {{ t("spaces.tags.selectionHint") }}
    </div>
    <template v-else>
      <div class="relative">
        <UiInput
          v-model="draft"
          :disabled="saving || !nodeOperations.allows('edit')"
          size="sm"
          class="pr-8"
          :placeholder="t('spaces.tags.add')"
          :aria-label="t('spaces.tags.add')"
          @keydown.enter.prevent="applyTag()"
          @keydown.esc="draft = ''"
        />
        <UiIconButton
          :label="t('spaces.tags.add')"
          size="sm"
          class="absolute top-0 right-0"
          :disabled="saving || !draft.trim() || !nodeOperations.allows('edit')"
          @click="applyTag()"
        >
          <Plus class="size-3.5" />
        </UiIconButton>
        <div
          v-if="suggestions.length"
          class="absolute top-full left-0 z-20 mt-1 w-full rounded-panel bg-surface-overlay p-1 shadow-popover ring-1 ring-stroke-default"
        >
          <button
            v-for="[tag, count] in suggestions"
            :key="tag"
            type="button"
            :disabled="saving || !nodeOperations.allows('edit')"
            class="flex h-interaction w-full items-center justify-between rounded-md px-2 text-left text-caption hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
            @click="applyTag(tag)"
          >
            <span class="truncate">{{ tag }}</span>
            <span class="text-caption-sm text-fg-tertiary">{{ count }}</span>
          </button>
        </div>
      </div>
      <div v-if="selectionTagCounts.size" class="mt-2 flex flex-wrap gap-1">
        <span
          v-for="[tag, count] in selectionTagCounts"
          :key="tag"
          class="inline-flex h-(--knovum-control-height-sm) items-center gap-1 rounded-md bg-surface-control pr-1 pl-2 text-caption text-fg-secondary"
        >
          {{ tag }}
          <span v-if="count !== selectedNodes.length" class="text-caption-sm text-fg-tertiary"
            >{{ count }}/{{ selectedNodes.length }}</span
          >
          <button
            type="button"
            :disabled="saving || !nodeOperations.allows('edit')"
            class="grid size-5 place-items-center rounded-md hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
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
