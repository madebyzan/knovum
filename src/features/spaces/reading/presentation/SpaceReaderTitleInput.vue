<script setup lang="ts">
import { Pencil } from "@lucide/vue";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { useNodeDraft } from "../../inspection/presentation/use-node-draft";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const store = useSpaceStore();
const nodeDraft = useNodeDraft(computed(() => props.node.id));
const titleInput = useTemplateRef<InstanceType<typeof UiInput>>("titleInput");
const draft = shallowRef(props.node.data.title);
const editing = shallowRef(false);
const saving = shallowRef(false);

watch(
  () => props.node.data.title,
  (value) => (draft.value = value),
);
watch(
  () => props.node.id,
  () => (editing.value = false),
);

async function beginRename() {
  if (!nodeDraft.begin()) return;
  editing.value = true;
  await nextTick();
  titleInput.value?.focus();
}

function cancelRename() {
  if (!editing.value || saving.value) return;
  editing.value = false;
  nodeDraft.end();
  draft.value = props.node.data.title;
}

async function submitRename() {
  if (!editing.value || saving.value) return;
  const value = draft.value.trim();
  if (!value || value === props.node.data.title) {
    cancelRename();
    return;
  }
  saving.value = true;
  try {
    await store.saveNodeContent({
      id: props.node.id,
      expectedContent: props.node.data,
      data: { ...props.node.data, title: value },
    });
    editing.value = false;
    nodeDraft.end();
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <UiInput
    v-if="editing"
    ref="titleInput"
    v-model="draft"
    size="sm"
    :disabled="saving"
    class="min-w-0 flex-1 text-label font-semibold"
    @blur="submitRename"
    @commit.prevent="submitRename"
    @cancel.prevent="cancelRename"
  />
  <button
    v-else
    type="button"
    class="group/title flex min-w-0 items-center gap-1 rounded-md px-1.5 py-0.5 text-left transition-colors hover:bg-surface-control-hover disabled:cursor-default disabled:hover:bg-transparent"
    :disabled="!nodeDraft.available.value"
    :aria-label="nodeDraft.available.value ? t('spaces.inspector.renameHint') : undefined"
    :title="node.data.title"
    @click="beginRename"
  >
    <h2 class="truncate text-label font-semibold tracking-title text-fg-primary">{{ node.data.title }}</h2>
    <Pencil
      v-if="nodeDraft.available.value"
      class="size-3 shrink-0 text-fg-tertiary opacity-0 transition-opacity duration-120 group-hover/title:opacity-100"
    />
  </button>
</template>
