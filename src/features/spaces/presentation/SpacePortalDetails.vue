<script setup lang="ts">
import { computed, ref, toRaw, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpacePortalPayload } from "../model/nodes/node-payload";
import { spaceTargetOptions } from "../model/space";
import { useSpaceStore } from "../session/space-store";
import { useNodeDraft } from "./use-node-draft";

const props = defineProps<{ node: SpaceCanvasNode }>();
const store = useSpaceStore();
const nodeDraft = useNodeDraft(
  computed(() => props.node.id),
  "portal",
);
const { t } = useI18n();
const draft = ref<SpacePortalPayload>();
const options = computed(() => spaceTargetOptions(store.spaces, t("spaces.space.inbox")));
const dirty = computed(() =>
  props.node.data.kind === "portal" && draft.value
    ? draft.value.title !== props.node.data.title ||
      draft.value.targetSpaceId !== props.node.data.targetSpaceId
    : false,
);
watch(
  () => props.node.data,
  () => {
    if (props.node.data.kind === "portal") draft.value = structuredClone(toRaw(props.node.data));
  },
  { immediate: true, deep: true },
);
async function save() {
  if (!draft.value || !dirty.value || !nodeDraft.canContinue.value) return;
  try {
    await store.saveNodeData(props.node.id, props.node.data, draft.value);
    nodeDraft.end();
  } catch (error) {
    toast.error(String(error));
  }
}
function cancel() {
  if (props.node.data.kind === "portal") draft.value = structuredClone(toRaw(props.node.data));
  nodeDraft.end();
}
</script>

<template>
  <section v-if="draft" class="space-y-2" @focusin="nodeDraft.begin()" @focusout="!dirty && nodeDraft.end()">
    <UiInput
      v-model="draft.title"
      :disabled="!nodeDraft.canContinue.value"
      :placeholder="t('common.title')"
    />
    <UiSelect
      v-model="draft.targetSpaceId"
      :disabled="!nodeDraft.canContinue.value"
      :label="t('spaces.inspector.targetSpace')"
      :options="options"
    />
    <div v-if="dirty" class="flex gap-2">
      <UiButton size="sm" variant="ghost" @click="cancel">{{ t("common.cancel") }}</UiButton>
      <UiButton size="sm" variant="primary" :disabled="!nodeDraft.canContinue.value" @click="save">{{
        t("common.save")
      }}</UiButton>
    </div>
  </section>
</template>
