<script setup lang="ts">
import type { GenerationActionOption } from "../generation-action-option";

const props = defineProps<{ loadGenerationActions: () => Promise<GenerationActionOption[]> }>();
import { Copy, Pencil, Trash2 } from "@lucide/vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { camelCase } from "es-toolkit";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { GenerationPreset } from "../../../foundation/ai/schema/models";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import GenerationPresetEditor from "./GenerationPresetEditor.vue";
import SettingsSection from "./SettingsSection.vue";

const presets = defineModel<GenerationPreset[]>({ required: true });
const { t } = useI18n();
const editingIndex = shallowRef<number>();
const editorOpen = shallowRef(false);

async function removePreset(index: number) {
  const confirmed = await ask(t("settings.creation.presets.deleteConfirm"), {
    title: t("common.delete"),
    kind: "warning",
    okLabel: t("common.delete"),
    cancelLabel: t("common.cancel"),
  });
  if (confirmed) presets.value.splice(index, 1);
}

function duplicatePreset(index: number) {
  const source = presets.value[index]!;
  presets.value.splice(index + 1, 0, {
    ...structuredClone(source),
    id: crypto.randomUUID(),
    name: t("settings.creation.copyName", { name: source.name }),
  });
}

function editPreset(index: number) {
  editingIndex.value = index;
  editorOpen.value = true;
}

function savePreset(preset: GenerationPreset) {
  if (editingIndex.value !== undefined) presets.value.splice(editingIndex.value, 1, preset);
}
</script>

<template>
  <SettingsSection
    id="settings-generation-presets"
    tabindex="-1"
    :title="t('settings.creation.presets.title')"
    :description="t('settings.creation.presets.description')"
  >
    <p v-if="presets.length === 0" class="px-4 py-3 text-caption-md text-fg-secondary">
      {{ t("settings.creation.presets.empty") }}
    </p>
    <div v-else class="divide-y divide-stroke-divider">
      <div v-for="(preset, index) in presets" :key="preset.id" class="flex items-center gap-3 px-4 py-3">
        <div class="min-w-0 flex-1">
          <UiInput v-model="preset.name" :aria-label="t('settings.creation.presets.name')" />
          <p class="mt-0.5 text-caption-md text-fg-secondary">
            {{ t(`settings.creation.knowledgeActions.${camelCase(preset.actionId)}`) }}
          </p>
        </div>
        <UiIconButton :label="t('common.duplicate')" size="sm" @click="duplicatePreset(index)">
          <Copy class="size-3.5" />
        </UiIconButton>
        <UiIconButton :label="t('common.edit')" size="sm" @click="editPreset(index)">
          <Pencil class="size-3.5" />
        </UiIconButton>
        <UiIconButton :label="t('common.delete')" size="sm" @click="removePreset(index)">
          <Trash2 class="size-3.5" />
        </UiIconButton>
      </div>
    </div>
    <GenerationPresetEditor
      v-if="editingIndex !== undefined && presets[editingIndex]"
      v-model:open="editorOpen"
      :preset="presets[editingIndex]!"
      :load-generation-actions="props.loadGenerationActions"
      @save="savePreset"
    />
  </SettingsSection>
</template>
