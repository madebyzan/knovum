<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { camelCase } from "es-toolkit";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiInput from "../../../../shared/ui/UiInput.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import { nodeIcon } from "../node-icon";

const props = defineProps<{ node: SpaceCanvasNode }>();
const emit = defineEmits<{ titleChange: [value: string] }>();
const { t } = useI18n();
const draft = ref(props.node.data.title);
const editing = ref(false);

watch(
  () => props.node.data.title,
  (value) => {
    draft.value = value;
  },
);

const typeLabel = computed(() => t(`spaces.nodeTypes.${camelCase(props.node.data.kind)}`));
const previewUrl = computed(() => {
  const data = props.node.data;
  const previewPath = isSpaceManagedAssetPayload(data) ? data.previewPath : undefined;
  return previewPath ? convertFileSrc(previewPath) : undefined;
});
const kindIcon = computed(() => nodeIcon(props.node.data.kind));

function submit() {
  const value = draft.value.trim();
  editing.value = false;
  if (!value || value === props.node.data.title) {
    draft.value = props.node.data.title;
    return;
  }
  emit("titleChange", value);
}

function cancel() {
  editing.value = false;
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
        <button v-if="!editing" type="button" class="min-w-0 flex-1 text-left" @click="editing = true">
          <h2 class="truncate text-body-lg font-semibold tracking-title text-foreground">
            {{ node.data.title }}
          </h2>
        </button>
        <UiInput
          v-else
          v-model="draft"
          size="sm"
          class="text-body-lg font-semibold tracking-title"
          @blur="submit"
          @keydown.enter.prevent="submit"
          @keydown.esc.prevent="cancel"
        />
      </div>
      <div class="flex items-center gap-2">
        <span class="text-micro text-foreground-tertiary">{{ typeLabel }}</span>
        <span v-if="node.data.locked" class="text-micro text-foreground-tertiary">{{
          t("spaces.canvas.lock")
        }}</span>
      </div>
    </div>
  </div>
</template>
