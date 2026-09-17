<script setup lang="ts">
import { Ellipsis, Pencil, Trash2 } from "@lucide/vue";
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";

const props = defineProps<{ title: string; disabled?: boolean }>();
const emit = defineEmits<{ rename: [title: string]; delete: [] }>();
const { t } = useI18n();
const renameOpen = ref(false);
const draft = ref(props.title);

watch(
  () => props.title,
  (value) => {
    if (!renameOpen.value) draft.value = value;
  },
);
function openRename() {
  draft.value = props.title;
  renameOpen.value = true;
}
function saveRename() {
  const title = draft.value.trim();
  if (!title) return;
  emit("rename", title);
  renameOpen.value = false;
}
</script>

<template>
  <UiDropdownMenu
    align="end"
    :side-offset="6"
    :items="[
      { label: t('spaces.assistant.renameKnowledgeThread'), icon: Pencil, onSelect: openRename },
      {
        label: t('spaces.assistant.deleteKnowledgeThread'),
        icon: Trash2,
        destructive: true,
        onSelect: () => emit('delete'),
      },
    ]"
  >
    <template #trigger>
      <UiIconButton :label="t('common.more')" size="sm" :disabled="disabled">
        <Ellipsis class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
    </template>
  </UiDropdownMenu>

  <UiDialog v-model:open="renameOpen" :title="t('spaces.assistant.renameKnowledgeThread')">
    <UiInput
      v-model="draft"
      :placeholder="t('spaces.assistant.knowledgeThreadName')"
      @keydown.enter.prevent="saveRename"
    />
    <template #footer>
      <UiButton variant="ghost" @click="renameOpen = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!draft.trim()" @click="saveRename">{{
        t("common.save")
      }}</UiButton>
    </template>
  </UiDialog>
</template>
