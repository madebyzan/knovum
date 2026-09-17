<script setup lang="ts" generic="T extends { id: string; name: string }">
import { storeToRefs } from "pinia";
import { computed, onMounted, shallowRef } from "vue";
import { toast } from "vue-sonner";

import { useCreationLibraryStore } from "../../../../foundation/ai/creation-library-store";
import type { CreationLibrary } from "../../../../foundation/ai/integration-settings";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";

const props = defineProps<{
  label: string;
  emptyLabel: string;
  placeholder: string;
  saveLabel: string;
  savedSuccessMessage: string;
  filterItems: (settings: CreationLibrary) => T[];
  createItem: () => T | undefined;
  addItemToSettings: (settings: CreationLibrary, item: T) => void;
  saveDisabled: boolean;
}>();

const emit = defineEmits<{
  apply: [item: T];
}>();

const store = useCreationLibraryStore();
const { library, ready, saveError } = storeToRefs(store);
const selected = shallowRef("");
const items = computed(() => props.filterItems(library.value));
const options = computed(() => items.value.map(({ id, name }) => ({ value: id, label: name })));

onMounted(store.ensureLoaded);

function apply(id: string) {
  selected.value = id;
  const item = items.value.find((candidate) => candidate.id === id);
  if (item) emit("apply", structuredClone(item));
}

async function save() {
  if (!ready.value || props.saveDisabled) return;
  const item = props.createItem();
  if (!item) return;
  props.addItemToSettings(library.value, item);
  await store.flush();
  if (saveError.value) {
    toast.error(String(saveError.value));
    return;
  }
  selected.value = item.id;
  toast.success(props.savedSuccessMessage);
}
</script>

<template>
  <div class="flex items-end gap-2">
    <label class="grid min-w-0 flex-1 gap-1.5 text-caption text-foreground-secondary">
      <span>{{ label }}</span>
      <UiSelect
        :model-value="selected"
        :label="label"
        :options="options"
        :disabled="!ready || items.length === 0"
        :placeholder="items.length === 0 ? emptyLabel : placeholder"
        @update:model-value="apply"
      />
    </label>
    <UiButton variant="ghost" :disabled="!ready || saveDisabled" @click="save">
      {{ saveLabel }}
    </UiButton>
  </div>
</template>
