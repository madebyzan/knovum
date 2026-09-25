<script setup lang="ts">
import { ArrowUpRight, LocateFixed } from "@lucide/vue";
import { camelCase } from "es-toolkit";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import { isSpaceNodeOpenable, openSpaceNode, selectAndRevealSpaceNode } from "../canvas/selection-actions";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { useSpaceStore } from "../session/space-store";
import { nodeIcon } from "./node-icon";
import { useNodeDraft } from "./use-node-draft";

const props = defineProps<{
  node: SpaceCanvasNode;
  saveTitle: (node: SpaceCanvasNode, value: string) => Promise<void>;
}>();
const { t } = useI18n();
const store = useSpaceStore();
const draft = shallowRef(props.node.data.title);
const editing = shallowRef(false);
const saving = shallowRef(false);
const nodeDraft = useNodeDraft(
  computed(() => props.node.id),
  "title",
);
const typeLabel = computed(() => t(`spaces.nodeTypes.${camelCase(props.node.data.kind)}`));
const kindIcon = computed(() => nodeIcon(props.node.data.kind));
const modifiedAt = computed(() => {
  const data = props.node.data;
  return "modifiedAt" in data && typeof data.modifiedAt === "number"
    ? new Date(data.modifiedAt).toLocaleString()
    : undefined;
});

watch(
  () => props.node.data.title,
  (value) => (draft.value = value),
);
watch(
  () => props.node.id,
  () => (editing.value = false),
);

function beginEdit() {
  if (nodeDraft.begin()) editing.value = true;
}

async function submit() {
  if (!editing.value || saving.value) return;
  const value = draft.value.trim();
  if (!value || value === props.node.data.title) {
    cancel();
    return;
  }
  saving.value = true;
  try {
    await props.saveTitle(props.node, value);
    editing.value = false;
    nodeDraft.end();
  } catch (error) {
    toast.error(String(error));
  } finally {
    saving.value = false;
  }
}

function cancel() {
  if (!editing.value || saving.value) return;
  editing.value = false;
  nodeDraft.end();
  draft.value = props.node.data.title;
}

async function open() {
  try {
    await openSpaceNode(store, props.node);
  } catch (error) {
    toast.error(String(error));
  }
}
</script>

<template>
  <div class="px-4 pt-5 pb-4">
    <div class="flex items-center gap-1.5 text-caption text-foreground-tertiary">
      <component :is="kindIcon" class="size-(--knovum-icon-size-xs)" :stroke-width="1.6" />
      <span>{{ typeLabel }}</span>
    </div>
    <button
      v-if="!editing"
      type="button"
      class="mt-2 block w-full text-left disabled:cursor-default"
      :disabled="!nodeDraft.available.value"
      :title="t('spaces.inspector.renameHint')"
      @click="beginEdit"
    >
      <h2 class="text-title leading-snug font-semibold tracking-title wrap-break-word text-foreground">
        {{ node.data.title }}
      </h2>
    </button>
    <UiInput
      v-else
      v-model="draft"
      :disabled="saving"
      class="mt-2 w-full text-title font-semibold"
      @blur="submit"
      @keydown.enter.prevent="submit"
      @keydown.esc.prevent="cancel"
    />
    <p v-if="modifiedAt" class="mt-1 text-caption text-foreground-tertiary">
      {{ t("spaces.inspector.detailModified") }} · {{ modifiedAt }}
    </p>
    <div class="mt-3 flex flex-wrap items-center gap-1.5">
      <UiButton v-if="isSpaceNodeOpenable(node)" size="sm" @click="open">
        {{ t("spaces.inspector.openContent") }}
        <ArrowUpRight class="size-(--knovum-icon-size-xs)" />
      </UiButton>
      <UiButton size="sm" variant="ghost" @click="selectAndRevealSpaceNode(node.id)">
        <LocateFixed class="size-(--knovum-icon-size-xs)" />
        {{ t("spaces.inspector.locateOnCanvas") }}
      </UiButton>
    </div>
  </div>
</template>
