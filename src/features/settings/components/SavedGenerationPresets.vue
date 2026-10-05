<script setup lang="ts">
import { Copy, Pencil, Trash2 } from "@lucide/vue";
import { camelCase } from "es-toolkit";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { GenerationPreset } from "../../../foundation/ai/schema/models";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import type { GenerationActionOption } from "../generation-action-option";
import GenerationPresetEditor from "./GenerationPresetEditor.vue";
import SettingsSection from "./SettingsSection.vue";

const props = defineProps<{
  presets: GenerationPreset[];
  loadGenerationActions: () => Promise<GenerationActionOption[]>;
}>();
const emit = defineEmits<{
  remove: [id: string];
  duplicate: [id: string, name: string];
  rename: [id: string, name: string];
  save: [preset: GenerationPreset];
}>();
const { t } = useI18n();
const editingId = shallowRef<string>();
const editingPreset = computed(() => props.presets.find((preset) => preset.id === editingId.value));
const editorOpen = shallowRef(false);

async function removePreset(id: string) {
  const confirmed = await confirmAction({
    title: t("common.delete"),
    description: t("settings.creation.presets.deleteConfirm"),
    confirmLabel: t("common.delete"),
    danger: true,
  });
  if (confirmed) emit("remove", id);
}

function duplicatePreset(source: GenerationPreset) {
  emit("duplicate", source.id, t("settings.creation.copyName", { name: source.name }));
}

function editPreset(id: string) {
  editingId.value = id;
  editorOpen.value = true;
}
</script>

<template>
  <SettingsSection
    id="settings-generation-presets"
    tabindex="-1"
    :title="t('settings.creation.presets.title')"
    :description="t('settings.creation.presets.description')"
  >
    <p v-if="presets.length === 0" class="px-4 py-3 text-caption text-fg-secondary">
      {{ t("settings.creation.presets.empty") }}
    </p>
    <div v-else class="divide-y divide-stroke-divider">
      <div v-for="preset in presets" :key="preset.id" class="flex items-center gap-3 px-4 py-3">
        <div class="min-w-0 flex-1">
          <UiInput
            :model-value="preset.name"
            :aria-label="t('settings.creation.presets.name')"
            @update:model-value="emit('rename', preset.id, $event)"
          />
          <p class="mt-0.5 text-caption text-fg-secondary">
            {{ t(`settings.creation.knowledgeActions.${camelCase(preset.actionId)}`) }}
          </p>
        </div>
        <UiIconButton :label="t('common.duplicate')" size="sm" @click="duplicatePreset(preset)">
          <Copy class="size-3.5" />
        </UiIconButton>
        <UiIconButton :label="t('common.edit')" size="sm" @click="editPreset(preset.id)">
          <Pencil class="size-3.5" />
        </UiIconButton>
        <UiIconButton :label="t('common.delete')" size="sm" @click="removePreset(preset.id)">
          <Trash2 class="size-3.5" />
        </UiIconButton>
      </div>
    </div>
    <GenerationPresetEditor
      v-if="editingPreset"
      v-model:open="editorOpen"
      :preset="editingPreset"
      :load-generation-actions="props.loadGenerationActions"
      @save="emit('save', $event)"
    />
  </SettingsSection>
</template>
