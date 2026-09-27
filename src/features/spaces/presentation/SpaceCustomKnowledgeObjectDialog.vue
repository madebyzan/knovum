<script setup lang="ts">
import { isNotNil, omit } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { isSpaceSemanticNode } from "../knowledge/model/semantic-node";
import { useSpaceObjectTypesQuery } from "../knowledge/queries/use-knowledge-queries";
import type { SpaceKnowledgeObjectPayload } from "../model/nodes/node-payload";
import { useSpaceStore } from "../session/space-store";
import SpaceKnowledgeFieldEditor from "./knowledge/SpaceKnowledgeFieldEditor.vue";

const open = defineModel<boolean>("open", { required: true });
const store = useSpaceStore();
const operations = useNodeOperationGate();
const { activeSpaceId } = storeToRefs(store);
const { t } = useI18n();
const typesQuery = useSpaceObjectTypesQuery(activeSpaceId);
const typeId = ref("");
const title = ref("");
const summary = ref("");
type ObjectPropertyValue = SpaceKnowledgeObjectPayload["properties"][string];
const properties = ref<Record<string, ObjectPropertyValue>>({});
const customTypes = computed(() => (typesQuery.data.value ?? []).filter((type) => !type.system));
const typeOptions = computed(() => customTypes.value.map((type) => ({ value: type.id, label: type.name })));
const selectedType = computed(() => customTypes.value.find((type) => type.id === typeId.value));
const nodeOptions = computed(() =>
  store.nodes.filter(isSpaceSemanticNode).map((node) => ({ value: node.id, label: node.data.title })),
);
const valid = computed(() =>
  Boolean(
    title.value.trim() &&
    selectedType.value &&
    selectedType.value.fields.every((field) => !field.required || isNotNil(properties.value[field.key])),
  ),
);

function resetDraft() {
  typeId.value = customTypes.value[0]?.id ?? "";
  title.value = "";
  summary.value = "";
  properties.value = {};
}

watch([open, activeSpaceId], ([value]) => {
  if (value) resetDraft();
});
watch(customTypes, (types) => {
  if (open.value && !types.some((type) => type.id === typeId.value)) {
    typeId.value = types[0]?.id ?? "";
  }
});
watch(
  () => typesQuery.error.value,
  (error) => {
    if (error) toast.error(String(error));
  },
);
watch(typeId, () => {
  properties.value = {};
});
function setProperty(key: string, value: ObjectPropertyValue | undefined) {
  properties.value =
    value === undefined ? omit(properties.value, [key]) : { ...properties.value, [key]: value };
}
async function create() {
  if (!valid.value || !selectedType.value || !operations.allows("create")) return;
  try {
    await store.addContent({
      kind: "object",
      objectTypeId: selectedType.value.id,
      title: title.value.trim(),
      status: "draft",
      properties: properties.value,
      ...(summary.value.trim() ? { summary: summary.value.trim() } : {}),
    });
    open.value = false;
  } catch (error) {
    toast.error(String(error));
  }
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.knowledge.objects.create')"
    :description="t('spaces.knowledge.objects.createDescription')"
  >
    <p v-if="!typeOptions.length" class="text-caption-md text-fg-tertiary">
      {{ t("spaces.knowledge.types.none") }}
    </p>
    <div v-else class="space-y-3">
      <UiSelect v-model="typeId" :label="t('spaces.knowledge.types.label')" :options="typeOptions" />
      <UiInput v-model="title" :placeholder="t('common.title')" />
      <UiTextarea v-model="summary" :rows="2" :placeholder="t('spaces.knowledge.summary')" />
      <SpaceKnowledgeFieldEditor
        v-for="field in selectedType?.fields ?? []"
        :key="field.key"
        :field="field"
        v-bind="properties[field.key] === undefined ? {} : { modelValue: properties[field.key] }"
        :node-options="nodeOptions"
        @update:model-value="setProperty(field.key, $event)"
      />
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid || !operations.allows('create')" @click="create">{{
        t("common.create")
      }}</UiButton>
    </template>
  </UiDialog>
</template>
