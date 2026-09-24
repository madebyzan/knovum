<script setup lang="ts">
import { useId } from "reka-ui";
import { shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiInput from "../../../../shared/ui/UiInput.vue";

const props = defineProps<{ color: string }>();
const emit = defineEmits<{ change: [color: string] }>();
const { t } = useI18n();
const inputId = useId();
const errorId = `${inputId}-error`;
const draft = shallowRef(props.color);
const invalid = shallowRef(false);

watch(
  () => props.color,
  (color) => {
    draft.value = color;
    invalid.value = false;
  },
);

function commit() {
  const color = draft.value.trim().toUpperCase();
  invalid.value = !/^#[0-9A-F]{6}$/.test(color);
  if (invalid.value) return;
  draft.value = color;
  emit("change", color);
}

function pick(event: Event) {
  draft.value = (event.currentTarget as HTMLInputElement).value;
  commit();
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.stopPropagation();
    draft.value = props.color;
    invalid.value = false;
    (event.currentTarget as HTMLInputElement).blur();
  } else if (event.key === "Enter") {
    (event.currentTarget as HTMLInputElement).blur();
  }
}
</script>

<template>
  <div>
    <label class="mb-1.5 block text-caption text-foreground-secondary" :for="inputId">
      {{ t("spaces.media.fontColor") }}
    </label>
    <div
      class="flex items-center gap-2 rounded-control border border-border bg-field px-2"
      :class="invalid ? 'border-danger focus-within:border-danger' : 'focus-within:border-focus-ring'"
    >
      <input
        :value="color"
        type="color"
        class="size-7 shrink-0 cursor-pointer border-0 bg-transparent"
        :aria-label="t('spaces.media.fontColor')"
        @input="pick"
      />
      <UiInput
        :id="inputId"
        v-model="draft"
        class="border-0 bg-transparent font-mono shadow-none"
        size="sm"
        :aria-invalid="invalid || undefined"
        :aria-describedby="invalid ? errorId : undefined"
        @change="commit"
        @keydown="handleKeydown"
      />
    </div>
    <p v-if="invalid" :id="errorId" class="mt-1 text-micro text-danger">
      {{ t("spaces.media.invalidTextColor") }}
    </p>
  </div>
</template>
