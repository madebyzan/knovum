<script setup lang="ts">
import { nextTick, onMounted, useTemplateRef, watch } from "vue";

const model = defineModel<string>({ required: true });
defineProps<{
  disabled?: boolean;
  label: string;
  placeholder: string;
}>();
const emit = defineEmits<{ enterBody: []; blur: [] }>();
const textareaRef = useTemplateRef<HTMLTextAreaElement>("textareaRef");

function resize() {
  const el = textareaRef.value;
  if (!el) return;
  el.style.height = "auto";
  el.style.height = `${el.scrollHeight}px`;
}

watch(model, () => void nextTick(resize));
onMounted(() => void nextTick(resize));

function handleKeydown(event: KeyboardEvent) {
  if (event.isComposing) return;
  if (event.key === "Enter") {
    event.preventDefault();
    emit("enterBody");
    return;
  }
  if (event.key === "ArrowDown") {
    const el = textareaRef.value;
    if (el && el.selectionStart === el.value.length) {
      event.preventDefault();
      emit("enterBody");
    }
  }
}

defineExpose({
  focusEnd: () => {
    const el = textareaRef.value;
    if (!el) return;
    el.focus();
    const end = el.value.length;
    el.setSelectionRange(end, end);
  },
});
</script>

<template>
  <textarea
    ref="textareaRef"
    v-model="model"
    rows="1"
    :disabled="disabled"
    :aria-label="label"
    :placeholder="placeholder"
    class="block w-full resize-none overflow-hidden bg-transparent text-[2rem] leading-[1.22] font-bold tracking-[-0.025em] text-fg-primary outline-none placeholder:text-fg-tertiary/60"
    @input="resize"
    @keydown="handleKeydown"
    @blur="emit('blur')"
  />
</template>
