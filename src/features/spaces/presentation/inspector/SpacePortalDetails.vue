<script setup lang="ts">
import { computed, ref, toRaw, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpacePortalPayload } from "../../model/nodes/node-payload";
import { spaceTargetOptions } from "../../model/space";
import { useSpaceStore } from "../../session/space-store";

const props = defineProps<{ node: SpaceCanvasNode }>();
const store = useSpaceStore();
const { t } = useI18n();
const draft = ref<SpacePortalPayload>();
const options = computed(() => spaceTargetOptions(store.spaces, t("spaces.space.inbox")));
watch(
  () => props.node.data,
  () => {
    if (props.node.data.kind === "portal") draft.value = structuredClone(toRaw(props.node.data));
  },
  { immediate: true, deep: true },
);
async function save() {
  if (!draft.value) return;
  try {
    await store.saveNodeData(props.node.id, draft.value);
  } catch (error) {
    toast.error(String(error));
  }
}
</script>

<template>
  <section v-if="draft" class="space-y-2">
    <UiInput v-model="draft.title" :placeholder="t('spaces.inspector.detailTitle')" />
    <UiSelect v-model="draft.targetSpaceId" :label="t('spaces.inspector.targetSpace')" :options="options" />
    <UiButton size="sm" variant="primary" @click="save">{{ t("common.save") }}</UiButton>
  </section>
</template>
