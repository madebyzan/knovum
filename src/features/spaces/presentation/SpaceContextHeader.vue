<script setup lang="ts">
import { ArrowLeft, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../shared/ui/UiIconButton.vue";

defineProps<{ title: string; subtitle?: string; back?: boolean; closable?: boolean }>();
const emit = defineEmits<{ back: []; close: [] }>();
const { t } = useI18n();
</script>

<template>
  <header
    class="flex h-(--knovum-window-chrome-height) shrink-0 items-center gap-2 border-b border-window-hairline bg-panel px-3"
  >
    <UiIconButton v-if="back" :label="t('common.back')" size="sm" @click="emit('back')">
      <ArrowLeft class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
    </UiIconButton>
    <div class="min-w-0 flex-1">
      <h2 class="truncate text-body-lg font-semibold tracking-title">{{ title }}</h2>
      <p v-if="subtitle" class="mt-0.5 truncate text-micro text-foreground-tertiary">{{ subtitle }}</p>
    </div>
    <slot name="actions" />
    <UiIconButton v-if="closable" :label="t('common.close')" size="sm" @click="emit('close')">
      <X class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
    </UiIconButton>
  </header>
</template>
