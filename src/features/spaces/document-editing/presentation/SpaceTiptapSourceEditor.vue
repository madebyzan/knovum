<script setup lang="ts">
import { useTextareaAutosize } from "@vueuse/core";
import { useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

const model = defineModel<string>({ required: true });
defineProps<{ disabled?: boolean }>();
const emit = defineEmits<{ blur: [] }>();
const { t } = useI18n();
const textareaRef = useTemplateRef<HTMLTextAreaElement>("textareaRef");

useTextareaAutosize({ element: textareaRef, input: model });
</script>

<template>
  <textarea
    ref="textareaRef"
    v-model="model"
    data-testid="tiptap-markdown-source"
    :disabled="disabled"
    spellcheck="false"
    :aria-label="t('spaces.richtext.modes.source')"
    :placeholder="t('spaces.editor.placeholder')"
    class="mt-6 min-h-128 w-full resize-none overflow-hidden bg-transparent font-mono text-body leading-relaxed text-fg-primary outline-none placeholder:text-fg-tertiary"
    @blur="emit('blur')"
  />
</template>
