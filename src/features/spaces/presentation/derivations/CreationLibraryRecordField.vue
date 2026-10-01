<script setup lang="ts" generic="T extends { id: string; name: string }">
import { cloneDeep } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed, onMounted, shallowRef } from "vue";
import { toast } from "vue-sonner";

import { useCreationLibraryStore } from "../../../../foundation/ai/creation-library-store";
import type { CreationLibrary } from "../../../../foundation/ai/schema/creation-library";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";

const props = defineProps<{
  label: string;
  emptyLabel: string;
  placeholder: string;
  saveLabel: string;
  savedSuccessMessage: string;
  filterRecords: (library: CreationLibrary) => T[];
  createRecord: () => T | undefined;
  addRecordToLibrary: (library: CreationLibrary, record: T) => void;
  saveDisabled: boolean;
}>();

const emit = defineEmits<{
  apply: [record: T];
}>();

const store = useCreationLibraryStore();
const { library, ready, saveError } = storeToRefs(store);
const selected = shallowRef("");
const records = computed(() => props.filterRecords(library.value));
const options = computed(() => records.value.map(({ id, name }) => ({ value: id, label: name })));

onMounted(store.ensureLoaded);

function apply(id: string) {
  selected.value = id;
  const record = records.value.find((candidate) => candidate.id === id);
  if (record) emit("apply", cloneDeep(record));
}

async function save() {
  if (!ready.value || props.saveDisabled) return;
  const record = props.createRecord();
  if (!record) return;
  props.addRecordToLibrary(library.value, record);
  await store.flush();
  if (saveError.value) {
    toast.error(String(saveError.value));
    return;
  }
  selected.value = record.id;
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
        :disabled="!ready || records.length === 0"
        :placeholder="records.length === 0 ? emptyLabel : placeholder"
        @update:model-value="apply"
      />
    </label>
    <UiButton variant="ghost" :disabled="!ready || saveDisabled" @click="save">
      {{ saveLabel }}
    </UiButton>
  </div>
</template>
