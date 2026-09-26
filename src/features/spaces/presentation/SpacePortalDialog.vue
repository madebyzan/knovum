<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { spaceTargetOptions } from "../model/space";
import { useSpaceStore } from "../session/space-store";

const open = defineModel<boolean>("open", { required: true });
const store = useSpaceStore();
const operations = useNodeOperationGate();
const { t } = useI18n();
const targetSpaceId = ref("");
const title = ref("");
const options = computed(() => spaceTargetOptions(store.spaces, t("spaces.space.inbox")));

watch(open, (value) => {
  if (!value) return;
  targetSpaceId.value =
    options.value.find((option) => option.value !== store.activeSpaceId)?.value ??
    options.value[0]?.value ??
    "";
  title.value = options.value.find((option) => option.value === targetSpaceId.value)?.label ?? "";
});
watch(targetSpaceId, (value) => {
  const label = options.value.find((option) => option.value === value)?.label;
  if (label && !title.value.trim()) title.value = label;
});

async function create() {
  if (!targetSpaceId.value || !operations.allows("create")) return;
  try {
    await store.addContent({
      kind: "portal",
      title: title.value.trim() || t("spaces.nodeTypes.portal"),
      targetSpaceId: targetSpaceId.value,
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
    :title="t('spaces.knowledge.portal.create')"
    :description="t('spaces.knowledge.portal.createDescription')"
  >
    <div class="space-y-3">
      <UiSelect v-model="targetSpaceId" :label="t('spaces.inspector.targetSpace')" :options="options" />
      <UiInput v-model="title" :placeholder="t('common.title')" />
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton
        variant="primary"
        :disabled="!targetSpaceId || !operations.allows('create')"
        @click="create"
        >{{ t("common.create") }}</UiButton
      >
    </template>
  </UiDialog>
</template>
