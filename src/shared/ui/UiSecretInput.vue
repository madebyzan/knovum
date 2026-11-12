<script setup lang="ts">
import { Eye, EyeOff } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "./UiIconButton.vue";
import UiInput from "./UiInput.vue";

const model = defineModel<string>({ default: "" });
withDefaults(
  defineProps<{
    id?: string | undefined;
    ariaLabel?: string | undefined;
    placeholder?: string | undefined;
    disabled?: boolean | undefined;
  }>(),
  { id: undefined, ariaLabel: undefined, placeholder: "", disabled: false },
);
const emit = defineEmits<{ commit: [] }>();

const { t } = useI18n();
const reveal = shallowRef(false);
const dirty = shallowRef(false);
const canReveal = computed(() => model.value.length > 0);

function handleInput(value: string) {
  dirty.value = true;
  model.value = value;
}

function finish() {
  reveal.value = false;
  if (!dirty.value) return;
  dirty.value = false;
  emit("commit");
}
</script>

<template>
  <div class="relative">
    <UiInput
      :id="id"
      :model-value="model"
      :aria-label="ariaLabel"
      :type="reveal && canReveal ? 'text' : 'password'"
      :placeholder="placeholder"
      :disabled="disabled"
      autocomplete="off"
      class="pr-9"
      @update:model-value="handleInput"
      @blur="finish"
      @commit="finish"
    />
    <UiIconButton
      v-if="canReveal"
      :label="reveal ? t('common.hideSecret') : t('common.showSecret')"
      :disabled="disabled"
      size="sm"
      class="absolute top-1/2 right-1 -translate-y-1/2"
      @mousedown.prevent
      @click="reveal = !reveal"
    >
      <EyeOff v-if="reveal" class="size-3.5" />
      <Eye v-else class="size-3.5" />
    </UiIconButton>
  </div>
</template>
