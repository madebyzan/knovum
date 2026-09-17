<script setup lang="ts">
import { omit } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed, ref, toRaw, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { knowledgeObjectStatusesForType } from "../../knowledge/model/object-types";
import { isSpaceSemanticNode } from "../../knowledge/model/semantic-node";
import { useSpaceObjectTypesQuery } from "../../knowledge/queries/queries";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceKnowledgeObjectPayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import SpaceKnowledgeFieldEditor from "./SpaceKnowledgeFieldEditor.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const emit = defineEmits<{ cancel: []; saved: [] }>();
const store = useSpaceStore();
const { activeSpaceId } = storeToRefs(store);
const { t } = useI18n();
const typesQuery = useSpaceObjectTypesQuery(activeSpaceId);
const summary = ref("");
const status = ref("draft");
type ObjectPropertyValue = SpaceKnowledgeObjectPayload["properties"][string];
const properties = ref<Record<string, ObjectPropertyValue>>({});
const data = computed(() => props.node.data as SpaceKnowledgeObjectPayload);
const objectType = computed(() => typesQuery.data.value?.find((type) => type.id === data.value.objectTypeId));
const statusOptions = computed(() =>
  knowledgeObjectStatusesForType(data.value.objectTypeId).map((value) => ({
    value,
    label: t(`spaces.knowledge.knowledgeStatusLabel.${value}`),
  })),
);
const nodeOptions = computed(() =>
  store.nodes
    .filter((node) => node.id !== props.node.id && isSpaceSemanticNode(node))
    .map((node) => ({ value: node.id, label: node.data.title })),
);

watch(() => props.node.data, resetDraft, { deep: true, immediate: true });

function resetDraft() {
  summary.value = data.value.summary ?? "";
  status.value = data.value.status;
  properties.value = structuredClone(toRaw(data.value.properties));
}

function updateProperty(key: string, value: ObjectPropertyValue | undefined) {
  properties.value =
    value === undefined ? omit(properties.value, [key]) : { ...properties.value, [key]: value };
}

async function save() {
  const next: SpaceKnowledgeObjectPayload = {
    ...data.value,
    status: status.value as SpaceKnowledgeObjectPayload["status"],
    properties: properties.value,
    ...(summary.value.trim() ? { summary: summary.value.trim() } : {}),
  };
  if (!summary.value.trim()) delete next.summary;
  try {
    await store.saveNodeData(props.node.id, next);
    emit("saved");
  } catch (error) {
    toast.error(String(error));
  }
}
</script>

<template>
  <div v-if="objectType" class="space-y-3 px-1">
    <label class="block">
      <span class="mb-1 block text-micro text-foreground-tertiary">{{
        t("spaces.knowledge.knowledgeStatus")
      }}</span>
      <UiSelect v-model="status" :label="t('spaces.knowledge.knowledgeStatus')" :options="statusOptions" />
    </label>
    <label class="block">
      <span class="mb-1 block text-micro text-foreground-tertiary">{{
        t("spaces.knowledge.knowledgeSummary")
      }}</span>
      <UiTextarea v-model="summary" :rows="3" />
    </label>
    <SpaceKnowledgeFieldEditor
      v-for="field in objectType.fields"
      :key="field.key"
      :field="field"
      v-bind="properties[field.key] === undefined ? {} : { modelValue: properties[field.key] }"
      :node-options="nodeOptions"
      @update:model-value="updateProperty(field.key, $event)"
    />
    <div class="flex justify-end gap-1.5">
      <UiButton size="sm" variant="ghost" @click="emit('cancel')">{{ t("common.cancel") }}</UiButton>
      <UiButton size="sm" variant="primary" @click="save">{{ t("common.save") }}</UiButton>
    </div>
  </div>
</template>
