<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiIconButton from "../../shared/ui/UiIconButton.vue";
import UiPanelToggleIcon from "../../shared/ui/UiPanelToggleIcon.vue";

defineProps<{ sidebarPinned: boolean }>();
const emit = defineEmits<{
  "toggle-sidebar": [];
  "preview-sidebar": [];
  "cancel-sidebar-preview": [];
}>();
const { t } = useI18n();
</script>

<template>
  <div
    class="flex items-center"
    @pointerenter="emit('preview-sidebar')"
    @pointermove="emit('preview-sidebar')"
    @pointerleave="emit('cancel-sidebar-preview')"
  >
    <UiIconButton
      :label="t('shell.layout.toggleSidebar')"
      aria-controls="spaces-sidebar"
      :aria-expanded="sidebarPinned"
      shortcut="$mod+B"
      size="md"
      @click="emit('toggle-sidebar')"
    >
      <UiPanelToggleIcon class="size-4.5" side="left" :open="sidebarPinned" />
    </UiIconButton>
  </div>
</template>
