<script setup lang="ts">
import { PanelLeftClose, PanelLeftOpen } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../shared/ui/UiIconButton.vue";

const props = defineProps<{ sidebarPinned: boolean }>();
const emit = defineEmits<{
  "toggle-sidebar": [];
  "preview-sidebar": [];
  "cancel-sidebar-preview": [];
}>();
const { t } = useI18n();
const sidebarIcon = computed(() => (props.sidebarPinned ? PanelLeftClose : PanelLeftOpen));
</script>

<template>
  <div class="flex items-center">
    <div
      @pointerenter="emit('preview-sidebar')"
      @pointermove="emit('preview-sidebar')"
      @pointerleave="emit('cancel-sidebar-preview')"
    >
      <UiIconButton
        :label="t('shell.toggleSidebar')"
        shortcut="$mod+B"
        size="md"
        @click="emit('toggle-sidebar')"
      >
        <span class="relative size-(--knovum-icon-size-toolbar)">
          <Transition name="sidebar-icon">
            <component
              :is="sidebarIcon"
              :key="sidebarPinned ? 'close' : 'open'"
              class="absolute inset-0 size-(--knovum-icon-size-toolbar)"
              :stroke-width="1.55"
            />
          </Transition>
        </span>
      </UiIconButton>
    </div>
  </div>
</template>

<style scoped>
.sidebar-icon-enter-active,
.sidebar-icon-leave-active {
  transition:
    opacity 120ms ease-out,
    transform 140ms var(--ease-standard);
}
.sidebar-icon-enter-from {
  opacity: 0;
  transform: translate3d(2px, 0, 0);
}
.sidebar-icon-leave-to {
  opacity: 0;
  transform: translate3d(-2px, 0, 0);
}
@media (prefers-reduced-motion: reduce) {
  .sidebar-icon-enter-active,
  .sidebar-icon-leave-active {
    transition-duration: 0.01ms;
  }
}
</style>
