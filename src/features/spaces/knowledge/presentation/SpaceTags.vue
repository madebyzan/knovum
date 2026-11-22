<script setup lang="ts">
import { Plus, X } from "@lucide/vue";
import { union, without } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { useCanvasSelectionProjection } from "../../canvas/use-canvas-selection-projection";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { createTagChanges } from "../../model/nodes/tag-changes";
import { useSpaceStore } from "../../session/space-store";
import { selectedTagCounts, spaceTagCounts, suggestTags } from "./tag-projections";

const props = defineProps<{ targetNodes?: readonly SpaceCanvasNode[] | undefined }>();
const store = useSpaceStore();
const { nodes } = storeToRefs(store);
const nodeOperations = useNodeOperationGate();
const { selectedNodes } = useCanvasSelectionProjection(nodes);
const activeNodes = computed(() => props.targetNodes ?? selectedNodes.value);
const { t } = useI18n();
const draft = shallowRef("");
const saving = shallowRef(false);
const selectionTagCounts = computed(() => selectedTagCounts(activeNodes.value));
const tagCounts = computed(() => spaceTagCounts(nodes.value));
const suggestions = computed(() => suggestTags(draft.value, tagCounts.value, selectionTagCounts.value));

async function applyTag(tag = draft.value) {
  const value = tag.trim();
  const spaceId = store.activeSpaceId;
  const initialDraft = draft.value;
  if (!value || activeNodes.value.length === 0 || !spaceId) return;
  const saved = await updateTags((tags) => union(tags, [value]));
  if (saved && store.activeSpaceId === spaceId && draft.value === initialDraft) draft.value = "";
}

function removeTag(tag: string) {
  return updateTags((tags) => without(tags, tag));
}

async function updateTags(update: (tags: readonly string[]) => string[]) {
  if (saving.value || !nodeOperations.allows("edit")) return false;
  const changes = createTagChanges(activeNodes.value, update);
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
</script>

<template>
  <section>
    <div v-if="activeNodes.length === 0" class="px-1 text-caption text-fg-tertiary">
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
          @commit.prevent="applyTag()"
          @cancel.prevent="draft = ''"
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
      <div v-if="selectionTagCounts.size" class="mt-2 flex flex-wrap gap-1.5">
        <span
          v-for="[tag, count] in selectionTagCounts"
          :key="tag"
          class="inline-flex h-6 items-center gap-1 rounded-md bg-surface-control pr-1 pl-2 text-caption text-fg-primary"
        >
          <span>{{ tag }}</span>
          <span v-if="count !== activeNodes.length" class="text-caption-sm text-fg-tertiary tabular-nums"
            >{{ count }}/{{ activeNodes.length }}</span
          >
          <button
            type="button"
            :disabled="saving || !nodeOperations.allows('edit')"
            class="grid size-4 place-items-center rounded-sm text-fg-tertiary transition-colors duration-100 hover:bg-surface-danger-subtle hover:text-fg-danger focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:pointer-events-none disabled:opacity-40"
            :aria-label="t('spaces.tags.remove', { tag })"
            @click="removeTag(tag)"
          >
            <X class="size-2.5" :stroke-width="1.8" />
          </button>
        </span>
      </div>
    </template>
  </section>
</template>
