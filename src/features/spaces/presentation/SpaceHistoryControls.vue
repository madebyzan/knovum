<script setup lang="ts">
import { ChevronLeft, ChevronRight } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import { useSpaceStore } from "../session/space-store";

const store = useSpaceStore();
const { canNavigateBack, canNavigateForward, workspaceMode } = storeToRefs(store);
const { t } = useI18n();
</script>

<template>
  <div v-if="workspaceMode === 'space'" class="flex items-center gap-0.5">
    <UiIconButton
      :label="t('spaces.space.back')"
      shortcut="Alt+ArrowLeft"
      size="sm"
      :disabled="!canNavigateBack"
      @click="store.navigateSpaceHistory(-1)"
    >
      <ChevronLeft class="size-3.5" :stroke-width="1.6" />
    </UiIconButton>
    <UiIconButton
      :label="t('spaces.space.forward')"
      shortcut="Alt+ArrowRight"
      size="sm"
      :disabled="!canNavigateForward"
      @click="store.navigateSpaceHistory(1)"
    >
      <ChevronRight class="size-3.5" :stroke-width="1.6" />
    </UiIconButton>
  </div>
</template>
