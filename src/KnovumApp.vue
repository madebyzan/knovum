<script setup lang="ts">
import { arch, platform } from "@tauri-apps/plugin-os";
import { storeToRefs } from "pinia";
import { TooltipProvider } from "reka-ui";
import { computed, nextTick, onScopeDispose, provide, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { toast, Toaster } from "vue-sonner";

// oxlint-disable-next-line import/no-unassigned-import
import "vue-sonner/style.css";

import ApplicationUpdates from "./app/ApplicationUpdates.vue";
import DictationOverlay from "./features/dictation/DictationOverlay.vue";
import { showBackupMaintenanceNotice } from "./features/settings/backup/maintenance-notice";
import { useSpaceArchiveSettings } from "./features/spaces/assets/use-space-archive-settings";
import {
  registerVoiceLibraryRunNavigation,
  restoreResourceRuns,
} from "./features/spaces/runs/resource-run-observation";
import { useSpacesWorkspace } from "./features/spaces/workspace-definition";
import { useBootstrapStore } from "./foundation/bootstrap/bootstrap-store";
import { getErrorMessage } from "./foundation/errors/error-format";
import { usePreferencesStore } from "./foundation/preferences/preferences-store";
import { archiveSettingsKey } from "./shared/contracts/archive-settings";
import UiConfirmAction from "./shared/ui/UiConfirmAction.vue";
import AppShell from "./shell/AppShell.vue";
import BootstrapScreen from "./shell/BootstrapScreen.vue";

const bootstrap = useBootstrapStore();
const { t } = useI18n();
const preferences = usePreferencesStore();
const { status, failure } = storeToRefs(bootstrap);
const { theme } = storeToRefs(preferences);
const workspace = useSpacesWorkspace();
const router = useRouter();
const route = useRoute();
const workspaceActive = computed(() => route.name === "spaces");
onScopeDispose(
  registerVoiceLibraryRunNavigation(async () => {
    await router.push({ path: "/settings", query: { pane: "voices" } });
  }),
);
const dictationSupported =
  (platform() === "macos" && arch() === "aarch64") || (platform() === "windows" && arch() === "x86_64");
provide(archiveSettingsKey, useSpaceArchiveSettings());
let hasInitializedReadyServices = false;
watch(
  status,
  async (currentStatus) => {
    if (currentStatus !== "ready" || hasInitializedReadyServices) return;
    hasInitializedReadyServices = true;
    await nextTick();
    void restoreResourceRuns().catch((cause: unknown) => toast.error(getErrorMessage(cause)));
    await showBackupMaintenanceNotice(t);
  },
  { immediate: true },
);
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
      <AppShell
        :workspace="workspace"
        :workspace-active="workspaceActive"
        @open-settings="router.push('/settings')"
      />
      <ApplicationUpdates />
      <DictationOverlay v-if="dictationSupported" />
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
