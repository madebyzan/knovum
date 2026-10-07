<script setup lang="ts">
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
const name = defineModel<string>("name", { required: true });
const markdown = defineModel<string>("markdown", { required: true });
defineProps<{ readonly: boolean; existing: boolean; busy: boolean; files: string[] }>();
const emit = defineEmits<{ save: []; close: [] }>();
</script>

<template>
  <form class="space-y-4" @submit.prevent="emit('save')" data-testid="skill-content-editor">
    <label class="block space-y-1.5 text-caption text-fg-secondary"
      >{{ $t("extensions.skillName")
      }}<UiInput v-model="name" :disabled="busy || readonly || existing" data-testid="skill-name"
    /></label>
    <label class="block space-y-1.5 text-caption text-fg-secondary"
      >SKILL.md<UiTextarea
        v-model="markdown"
        :readonly="readonly"
        :disabled="busy"
        class="min-h-64 font-mono text-caption"
        data-testid="skill-markdown"
    /></label>
    <p v-if="readonly" class="text-caption text-fg-secondary">{{ $t("extensions.copyToEdit") }}</p>
    <p v-if="files.length" class="text-caption text-fg-tertiary">{{ files.join(" · ") }}</p>
    <div class="flex gap-2">
      <UiButton v-if="!readonly" variant="primary" type="submit" :disabled="busy || !name || !markdown">{{
        $t("extensions.save")
      }}</UiButton
      ><UiButton :disabled="busy" variant="ghost" @click="emit('close')">{{
        $t("extensions.close")
      }}</UiButton>
    </div>
  </form>
</template>
