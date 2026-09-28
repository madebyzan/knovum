<script setup lang="ts">
import { ArrowLeft, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../shared/ui/UiIconButton.vue";

defineProps<{ title: string; subtitle?: string | undefined; back?: boolean; closable?: boolean }>();
const emit = defineEmits<{ back: []; close: [] }>();
const { t } = useI18n();
</script>

<template>
  <header
    class="flex h-(--knovum-titlebar-height) shrink-0 items-center gap-2 border-b border-(--knovum-window-border-color) bg-surface-panel px-3"
  >
    <UiIconButton v-if="back" :label="t('common.back')" size="sm" @click="emit('back')">
      <ArrowLeft class="size-3.5" :stroke-width="1.55" />
    </UiIconButton>
    <slot name="leading" />
    <div class="min-w-0 flex-1">
      <h2 class="truncate text-title-sm font-semibold tracking-title">{{ title }}</h2>
      <p v-if="subtitle" class="mt-0.5 truncate text-caption-sm text-fg-tertiary">{{ subtitle }}</p>
    </div>
    <slot name="actions" />
    <UiIconButton v-if="closable" :label="t('common.close')" size="sm" @click="emit('close')">
      <X class="size-3.5" :stroke-width="1.55" />
    </UiIconButton>
  </header>
</template>
