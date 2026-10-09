<script setup lang="ts">
import { cloneDeep } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed, onMounted, shallowRef } from "vue";
import { toast } from "vue-sonner";

import { useCreationLibraryStore } from "../../../../foundation/ai/creation-library-store";
import type { GenerationPreset } from "../../../../foundation/ai/schema/models";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";

const props = defineProps<{
  label: string;
  emptyLabel: string;
  placeholder: string;
  saveLabel: string;
  savedSuccessMessage: string;
  actionId: string;
  createPreset: () => GenerationPreset | undefined;
  saveDisabled: boolean;
}>();

const emit = defineEmits<{
  apply: [preset: GenerationPreset];
}>();

const store = useCreationLibraryStore();
const { library, ready } = storeToRefs(store);
const selected = shallowRef("");
const presets = computed(() =>
  library.value.generationPresets.filter((preset) => preset.actionId === props.actionId),
);
const options = computed(() => presets.value.map(({ id, name }) => ({ value: id, label: name })));

onMounted(store.ensureLoaded);

function apply(presetId: string) {
  selected.value = presetId;
  const preset = presets.value.find((candidate) => candidate.id === presetId);
  if (preset) emit("apply", cloneDeep(preset));
}

async function save() {
  if (!ready.value || props.saveDisabled) return;
  const preset = props.createPreset();
  if (!preset) return;
  try {
    await store.addPreset(preset);
  } catch (error) {
    toast.error(getErrorMessage(error));
    return;
  }
  selected.value = preset.id;
  toast.success(props.savedSuccessMessage);
}
</script>

<template>
  <div class="flex items-end gap-2">
    <label class="grid min-w-0 flex-1 gap-1.5 text-caption text-fg-secondary">
      <span>{{ label }}</span>
      <UiSelect
        :model-value="selected"
        :label="label"
        :options="options"
        :disabled="!ready || presets.length === 0"
        :placeholder="presets.length === 0 ? emptyLabel : placeholder"
        @update:model-value="apply"
      />
    </label>
    <UiButton variant="ghost" :disabled="!ready || saveDisabled" @click="save">
      {{ saveLabel }}
    </UiButton>
  </div>
</template>
