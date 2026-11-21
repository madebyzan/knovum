<script setup lang="ts">
import { Check, ChevronDown, Plus } from "@lucide/vue";
import { union } from "es-toolkit";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { createTagChanges } from "../../model/nodes/tag-changes";
import { useSpaceStore } from "../../session/space-store";

const KEYWORD_LIMIT = 8;

const props = defineProps<{
  node: SpaceCanvasNode;
  keywords: readonly string[];
}>();

const { t } = useI18n();
const store = useSpaceStore();
const nodeOperations = useNodeOperationGate();
const expanded = shallowRef(false);
const saving = shallowRef(false);

const adoptedTags = computed(() => new Set(props.node.data.tags ?? []));
const canAdopt = computed(() => !saving.value && nodeOperations.allows("edit"));
const visibleKeywords = computed(() =>
  expanded.value ? props.keywords : props.keywords.slice(0, KEYWORD_LIMIT),
);

async function adoptKeyword(keyword: string) {
  const value = keyword.trim();
  if (!value || !canAdopt.value || adoptedTags.value.has(value)) return;
  const changes = createTagChanges([props.node], (tags) => union(tags, [value]));
  if (!changes.length) return;
  saving.value = true;
  try {
    await store.updateNodeContents(changes);
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="space-y-1.5">
    <h4 class="text-caption font-medium text-fg-tertiary">
      {{ t("spaces.knowledge.visual.keywords") }}
    </h4>
    <div class="flex flex-wrap gap-1">
      <template v-for="keyword in visibleKeywords" :key="keyword">
        <span
          v-if="adoptedTags.has(keyword)"
          class="inline-flex h-5 items-center gap-1 rounded-md border border-stroke-divider bg-surface-control/55 px-1.5 text-caption-sm text-fg-tertiary select-none"
        >
          <Check class="size-2.5 shrink-0" />
          <span>{{ keyword }}</span>
        </span>
        <button
          v-else
          type="button"
          :disabled="!canAdopt"
          :title="t('spaces.knowledge.visual.adoptKeyword', { keyword })"
          :aria-label="t('spaces.knowledge.visual.adoptKeyword', { keyword })"
          class="group inline-flex h-5 cursor-pointer items-center gap-1 rounded-md border border-dashed border-stroke-default px-1.5 text-caption-sm text-fg-secondary transition-colors duration-120 select-none hover:border-stroke-strong hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:cursor-default disabled:opacity-50"
          @click="adoptKeyword(keyword)"
        >
          <Plus class="size-2.5 shrink-0 text-fg-tertiary transition-colors group-hover:text-fg-primary" />
          <span>{{ keyword }}</span>
        </button>
      </template>
      <button
        v-if="keywords.length > KEYWORD_LIMIT"
        type="button"
        :aria-expanded="expanded"
        class="inline-flex h-5 cursor-pointer items-center gap-0.5 rounded-md bg-surface-control px-1.5 text-caption-sm font-medium text-fg-secondary transition-colors duration-150 select-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
        @click="expanded = !expanded"
      >
        <span>{{
          expanded ? t("common.collapse") : `+${keywords.length - KEYWORD_LIMIT} ${t("common.more")}`
        }}</span>
        <ChevronDown
          class="size-2.5 text-fg-tertiary transition-transform duration-150"
          :class="expanded && 'rotate-180'"
        />
      </button>
    </div>
  </div>
</template>
