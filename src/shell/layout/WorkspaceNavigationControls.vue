<script setup lang="ts">
import { PanelLeft } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../shared/ui/UiIconButton.vue";

defineProps<{ sidebarPinned: boolean }>();
const emit = defineEmits<{
  "toggle-sidebar": [];
  "preview-sidebar": [];
  "cancel-sidebar-preview": [];
}>();
const { t } = useI18n();
</script>

<template>
  <!--
    One glyph with a pressed state, the same contract the right-hand toggle follows. Cross-fading two
    different line glyphs inside an 18px box paints both strokes over each other mid-transition, which
    reads as a flash on every click.
  -->
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
      :active="sidebarPinned"
      size="md"
      @click="emit('toggle-sidebar')"
    >
      <PanelLeft class="size-4.5" :stroke-width="1.55" />
    </UiIconButton>
  </div>
</template>
