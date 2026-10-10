<script setup lang="ts">
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
const name = defineModel<string>("name", { required: true });
const markdown = defineModel<string>("markdown", { required: true });
defineProps<{ existing: boolean; busy: boolean; files: string[] }>();
const emit = defineEmits<{ save: []; close: [] }>();
</script>

<template>
  <form class="space-y-4" @submit.prevent="emit('save')" data-testid="skill-content-editor">
    <label class="block space-y-1.5 text-caption text-fg-secondary"
      >{{ $t("extensions.skills.name")
      }}<UiInput v-model="name" :disabled="busy || existing" data-testid="skill-name"
    /></label>
    <label class="block space-y-1.5 text-caption text-fg-secondary"
      >SKILL.md<UiTextarea
        v-model="markdown"
        :disabled="busy"
        class="min-h-64 font-mono text-caption"
        data-testid="skill-markdown"
    /></label>
    <p v-if="files.length" class="text-caption text-fg-tertiary">{{ files.join(" · ") }}</p>
    <div class="flex gap-2">
      <UiButton variant="primary" type="submit" :disabled="busy || !name || !markdown">{{
        $t("common.save")
      }}</UiButton
      ><UiButton :disabled="busy" variant="ghost" @click="emit('close')">{{ $t("common.close") }}</UiButton>
    </div>
  </form>
</template>
