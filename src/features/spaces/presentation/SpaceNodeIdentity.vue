<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { camelCase } from "es-toolkit";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiInput from "../../../shared/ui/UiInput.vue";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import { nodeIcon } from "./node-icon";
import { useNodeDraft } from "./use-node-draft";

const props = defineProps<{
  node: SpaceCanvasNode;
  saveTitle: (node: SpaceCanvasNode, value: string) => Promise<void>;
}>();
const { t } = useI18n();
const draft = ref(props.node.data.title);
const editing = ref(false);
const saving = ref(false);
const nodeDraft = useNodeDraft(
  computed(() => props.node.id),
  "title",
);

watch(
  () => props.node.data.title,
  (value) => {
    draft.value = value;
  },
);
watch(
  () => props.node.id,
  () => (editing.value = false),
);

function beginEdit() {
  if (nodeDraft.begin()) editing.value = true;
}

const typeLabel = computed(() => t(`spaces.nodeTypes.${camelCase(props.node.data.kind)}`));
const previewUrl = computed(() => {
  const data = props.node.data;
  const previewPath = isSpaceManagedAssetPayload(data) ? data.previewPath : undefined;
  return previewPath ? convertFileSrc(previewPath) : undefined;
});
const kindIcon = computed(() => nodeIcon(props.node.data.kind));

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
</script>

<template>
  <div class="flex items-center gap-2.5 px-3 py-3">
    <div
      class="relative shrink-0 overflow-hidden rounded-control border border-divider bg-control"
      :class="previewUrl ? 'size-10' : 'grid size-9 place-items-center'"
    >
      <img v-if="previewUrl" :src="previewUrl" alt="" class="size-full object-contain" />
      <component
        :is="kindIcon"
        v-else
        class="size-(--knovum-icon-size-md) text-foreground-tertiary"
        :stroke-width="1.5"
      />
    </div>
    <div class="min-w-0 flex-1">
      <div class="flex min-h-(--knovum-control-height-sm) items-center">
        <button
          v-if="!editing"
          type="button"
          class="min-w-0 flex-1 text-left disabled:cursor-not-allowed"
          :disabled="!nodeDraft.available.value"
          @click="beginEdit"
        >
          <h2 class="truncate text-body-lg font-semibold tracking-title text-foreground">
            {{ node.data.title }}
          </h2>
        </button>
        <UiInput
          v-else
          v-model="draft"
          :disabled="saving"
          size="sm"
          class="text-body-lg font-semibold tracking-title"
          @blur="submit"
          @keydown.enter.prevent="submit"
          @keydown.esc.prevent="cancel"
        />
      </div>
      <span class="text-micro text-foreground-tertiary">{{ typeLabel }}</span>
    </div>
  </div>
</template>
