<script setup lang="ts">
import { Check, Copy } from "@lucide/vue";
import { useTimeoutFn } from "@vueuse/core";
import { writeText } from "tauri-plugin-clipboard-api";
import { shallowRef, type HTMLAttributes } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiIconButton from "./UiIconButton.vue";

const props = withDefaults(
  defineProps<{
    text: string;
    label?: string | undefined;
    copiedLabel?: string | undefined;
    errorMessage?: string | undefined;
    size?: "sm" | "md" | undefined;
    iconClass?: HTMLAttributes["class"];
    class?: HTMLAttributes["class"];
  }>(),
  {
    size: "sm",
    iconClass: "size-3.5",
  },
);

const { t } = useI18n();
const copied = shallowRef(false);
const { start: resetCopied } = useTimeoutFn(() => (copied.value = false), 1600, { immediate: false });

async function copy() {
  try {
    await writeText(props.text);
    copied.value = true;
    resetCopied();
  } catch (error) {
    copied.value = false;
    const description = error instanceof Error ? error.message : String(error);
    toast.error(props.errorMessage ?? t("common.copy"), { description });
  }
}
</script>

<template>
  <UiIconButton
    :label="copied ? (copiedLabel ?? t('common.copied')) : (label ?? t('common.copy'))"
    tooltip
    :size="size"
    :class="props.class"
    @click="copy"
  >
    <Check v-if="copied" :class="[iconClass, 'text-fg-accent']" />
    <Copy v-else :class="iconClass" />
  </UiIconButton>
</template>
