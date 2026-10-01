<script setup lang="ts">
import { storeToRefs } from "pinia";
import { TooltipProvider } from "reka-ui";
import { provide } from "vue";
import { Toaster } from "vue-sonner";

// oxlint-disable-next-line import/no-unassigned-import
import "vue-sonner/style.css";

import { createSpaceArchiveSettings } from "./features/spaces/assets/archive-settings";
import { useSpacesWorkspace } from "./features/spaces/workspace-definition";
import { useBootstrapStore } from "./foundation/bootstrap/bootstrap-store";
import { usePreferencesStore } from "./foundation/preferences/preferences-store";
import { archiveSettingsKey } from "./shared/contracts/archive-settings";
import UiConfirmAction from "./shared/ui/UiConfirmAction.vue";
import AppShell from "./shell/AppShell.vue";
import BootstrapScreen from "./shell/BootstrapScreen.vue";

const bootstrap = useBootstrapStore();
const preferences = usePreferencesStore();
const { status, failure } = storeToRefs(bootstrap);
const { theme } = storeToRefs(preferences);
const workspace = useSpacesWorkspace();
provide(archiveSettingsKey, createSpaceArchiveSettings());
</script>

<template>
  <div class="relative h-full min-h-0 w-full min-w-0 overflow-hidden bg-surface-chrome">
    <TooltipProvider
      v-if="status === 'ready'"
      :delay-duration="550"
      :skip-delay-duration="120"
      :ignore-non-keyboard-focus="true"
      :disable-hoverable-content="true"
    >
      <AppShell :workspace="workspace" />
      <UiConfirmAction />
      <Toaster :theme="theme" position="bottom-right" :visible-toasts="4" close-button />
    </TooltipProvider>

    <Transition name="bootstrap-fade">
      <BootstrapScreen
        v-if="status !== 'ready'"
        class="absolute inset-0 z-50"
        :failure="status === 'failed' ? failure : undefined"
      />
    </Transition>
  </div>
</template>

<style scoped>
.bootstrap-fade-leave-active {
  transition: opacity 220ms var(--ease-standard);
  pointer-events: none;
}
.bootstrap-fade-leave-to {
  opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
  .bootstrap-fade-leave-active {
    transition-duration: 0.01ms;
  }
}
</style>
