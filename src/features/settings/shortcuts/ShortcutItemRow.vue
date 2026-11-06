<script setup lang="ts">
import UiButton from "../../../shared/ui/UiButton.vue";
import UiKbd from "../../../shared/ui/UiKbd.vue";
import PreferenceField from "../components/PreferenceField.vue";

defineProps<{
  id: string;
  title: string;
  description?: string | undefined;
  shortcut?: string | undefined;
  customized: boolean;
  editLabel: string;
  resetLabel: string;
  customizedLabel: string;
}>();

defineEmits<{
  (e: "edit"): void;
  (e: "reset"): void;
}>();
</script>

<template>
  <PreferenceField :id="`settings-shortcut-${id}`" :label="title" :description="description">
    <div class="flex items-center gap-2">
      <span
        v-if="customized"
        class="bg-brand-primary/10 text-brand-primary rounded px-1.5 py-0.5 text-caption-sm font-medium"
      >
        {{ customizedLabel }}
      </span>
      <UiKbd :shortcut="shortcut" variant="badge" />
      <UiButton variant="secondary" size="sm" @click="$emit('edit')">
        {{ editLabel }}
      </UiButton>
      <UiButton v-if="customized" variant="ghost" size="sm" @click="$emit('reset')">
        {{ resetLabel }}
      </UiButton>
    </div>
  </PreferenceField>
</template>
