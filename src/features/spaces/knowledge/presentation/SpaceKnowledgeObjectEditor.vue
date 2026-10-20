<script setup lang="ts">
import { cloneDeep, omit } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type {
  SpaceKnowledgeObjectPayload,
  SpaceKnowledgeObjectPropertyValue,
} from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import { knowledgeObjectStatusesForType } from "../model/object-types";
import { isSpaceSemanticNode } from "../model/semantic-node";
import { useSpaceObjectTypesQuery } from "../queries/use-space-object-types-query";
import SpaceKnowledgeFieldEditor from "./SpaceKnowledgeFieldEditor.vue";

const props = defineProps<{ nodeId: string; data: SpaceKnowledgeObjectPayload }>();
const emit = defineEmits<{ cancel: []; saved: [] }>();
const store = useSpaceStore();
const { activeSpaceId } = storeToRefs(store);
const { t } = useI18n();
const typesQuery = useSpaceObjectTypesQuery(activeSpaceId);
const summary = ref("");
const status = ref<SpaceKnowledgeObjectPayload["status"]>("draft");
const properties = ref<Record<string, SpaceKnowledgeObjectPropertyValue>>({});
const objectType = computed(() => typesQuery.data.value?.find((type) => type.id === props.data.objectTypeId));
const statusOptions = computed(() =>
  knowledgeObjectStatusesForType(props.data.objectTypeId).map((value) => ({
    value,
    label: t(`spaces.knowledge.status.labels.${value}`),
  })),
);
const nodeOptions = computed(() =>
  store.nodes
    .filter((node) => node.id !== props.nodeId && isSpaceSemanticNode(node))
    .map((node) => ({ value: node.id, label: node.data.title })),
);

watch(() => props.data, resetDraft, { deep: true, immediate: true });

function resetDraft() {
  summary.value = props.data.summary ?? "";
  status.value = props.data.status;
  properties.value = cloneDeep(props.data.properties);
}

function updateProperty(key: string, value: SpaceKnowledgeObjectPropertyValue | undefined) {
  properties.value =
    value === undefined ? omit(properties.value, [key]) : { ...properties.value, [key]: value };
}

async function save() {
  const next: SpaceKnowledgeObjectPayload = {
    ...props.data,
    status: status.value,
    properties: properties.value,
    ...(summary.value.trim() ? { summary: summary.value.trim() } : {}),
  };
  if (!summary.value.trim()) delete next.summary;
  try {
    await store.saveNodeContent({ id: props.nodeId, expectedContent: props.data, data: next });
    emit("saved");
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}
</script>

<template>
  <div v-if="objectType" class="space-y-3 px-1">
    <label class="block">
      <span class="mb-1 block text-caption-sm text-fg-tertiary">{{
        t("spaces.knowledge.status.label")
      }}</span>
      <UiSelect v-model="status" :label="t('spaces.knowledge.status.label')" :options="statusOptions" />
    </label>
    <label class="block">
      <span class="mb-1 block text-caption-sm text-fg-tertiary">{{ t("spaces.knowledge.summary") }}</span>
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
