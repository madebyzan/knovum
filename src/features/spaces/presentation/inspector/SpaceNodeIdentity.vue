<script setup lang="ts">
import { ArrowUpRight, LocateFixed, Pencil } from "@lucide/vue";
import { camelCase } from "es-toolkit";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { isSpaceNodeOpenable, openSpaceNode, selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { nodeIcon } from "../canvas/node-icon";
import { useNodeDraft } from "./use-node-draft";

const props = defineProps<{
  node: SpaceCanvasNode;
  saveTitle: (node: SpaceCanvasNode, value: string) => Promise<void>;
}>();
const { d, t } = useI18n();
const store = useSpaceStore();
const draft = shallowRef(props.node.data.title);
const titleInput = useTemplateRef<InstanceType<typeof UiInput>>("titleInput");
const editing = shallowRef(false);
const saving = shallowRef(false);
const nodeDraft = useNodeDraft(computed(() => props.node.id));
const typeLabel = computed(() => t(`spaces.nodeTypes.${camelCase(props.node.data.kind)}`));
const kindIcon = computed(() => nodeIcon(props.node.data.kind));
const modifiedAt = computed(() => {
  const data = props.node.data;
  return "modifiedAt" in data && typeof data.modifiedAt === "number"
    ? d(data.modifiedAt, "short")
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

async function beginEdit() {
  if (!nodeDraft.begin()) return;
  editing.value = true;
  await nextTick();
  titleInput.value?.focus();
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
    toast.error(getErrorMessage(error));
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
    toast.error(getErrorMessage(error));
  }
}
</script>

<template>
  <div class="px-4 pt-4 pb-4">
    <div class="flex items-center gap-1.5 text-caption font-medium text-fg-secondary">
      <component :is="kindIcon" class="size-3.5" :stroke-width="1.6" />
      <span>{{ typeLabel }}</span>
    </div>
    <button
      v-if="!editing"
      type="button"
      class="mt-2 flex w-full items-start gap-2 rounded-md text-left focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:cursor-default"
      :disabled="!nodeDraft.available.value"
      :aria-label="nodeDraft.available.value ? t('spaces.inspector.renameHint') : undefined"
      @click="beginEdit"
    >
      <h2
        class="min-w-0 flex-1 text-title leading-snug font-semibold tracking-title wrap-break-word text-fg-primary"
      >
        {{ node.data.title }}
      </h2>
      <Pencil v-if="nodeDraft.available.value" class="mt-1 size-3.5 shrink-0 text-fg-tertiary" />
    </button>
    <UiInput
      v-else
      ref="titleInput"
      v-model="draft"
      :disabled="saving"
      class="mt-2 w-full text-title font-semibold"
      @blur="submit"
      @commit.prevent="submit"
      @cancel.prevent="cancel"
    />
    <p v-if="modifiedAt" class="mt-1 text-caption text-fg-tertiary">
      {{ t("spaces.inspector.fields.modified") }} · {{ modifiedAt }}
    </p>
    <div class="mt-3 flex flex-wrap items-center gap-1.5">
      <UiButton v-if="isSpaceNodeOpenable(node)" size="sm" @click="open">
        {{ t("spaces.inspector.openContent") }}
        <ArrowUpRight class="size-3" />
      </UiButton>
      <UiButton size="sm" variant="ghost" @click="selectAndRevealSpaceNode(node.id)">
        <LocateFixed class="size-3" />
        {{ t("spaces.canvas.revealInCanvas") }}
      </UiButton>
    </div>
  </div>
</template>
