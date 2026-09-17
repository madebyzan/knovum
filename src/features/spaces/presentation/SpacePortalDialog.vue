<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import { spaceTargetOptions } from "../model/space";
import { useSpaceStore } from "../session/space-store";

const open = defineModel<boolean>("open", { required: true });
const store = useSpaceStore();
const { t } = useI18n();
const targetSpaceId = ref("");
const title = ref("");
const options = computed(() => spaceTargetOptions(store.spaces, t("spaces.space.inbox")));

watch(open, (value) => {
  if (!value) return;
  targetSpaceId.value =
    options.value.find((item) => item.value !== store.activeSpaceId)?.value ?? options.value[0]?.value ?? "";
  title.value = options.value.find((item) => item.value === targetSpaceId.value)?.label ?? "";
});
watch(targetSpaceId, (value) => {
  const label = options.value.find((item) => item.value === value)?.label;
  if (label && !title.value.trim()) title.value = label;
});

async function create() {
  if (!targetSpaceId.value) return;
  try {
    await store.addItem({
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
    :title="t('spaces.knowledge.newPortal')"
    :description="t('spaces.knowledge.newPortalDescription')"
  >
    <div class="space-y-3">
      <UiSelect v-model="targetSpaceId" :label="t('spaces.inspector.targetSpace')" :options="options" />
      <UiInput v-model="title" :placeholder="t('spaces.inspector.detailTitle')" />
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!targetSpaceId" @click="create">{{
        t("common.create")
      }}</UiButton>
    </template>
  </UiDialog>
</template>
