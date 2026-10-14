import { error as logError } from "@tauri-apps/plugin-log";
import { pick } from "es-toolkit";
import { createRouter, createWebHashHistory } from "vue-router";
import { toast } from "vue-sonner";

import AgentExtensionsSettings from "../features/extensions/AgentExtensionsSettings.vue";
import SettingsView from "../features/settings/SettingsView.vue";
import { useSpaceStore } from "../features/spaces/session/space-store";
import SpaceView from "../features/spaces/SpaceView.vue";
import VoiceLibrarySettings from "../features/spaces/speech/presentation/VoiceLibrarySettings.vue";
import { listSpaceActions } from "../features/spaces/workflows/action-catalog";
import { formatErrorDiagnostic, getErrorMessage } from "../foundation/errors/error-format";
import { isApplicationLeaveCancelled } from "../shared/contracts/application-leave";
import { returnToWorkspace } from "./workspace-session";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/spaces" },
    {
      path: "/spaces",
      name: "spaces",
      component: SpaceView,
    },
    {
      path: "/settings",
      name: "settings",
      component: SettingsView,
      props: () => ({
        voiceLibraryComponent: VoiceLibrarySettings,
        extensionsComponent: AgentExtensionsSettings,
        returnToWorkspace: () => returnToWorkspace(router),
        loadGenerationActions: async () =>
          (await listSpaceActions()).map((action) => pick(action, ["id", "routingPurpose"])),
      }),
    },
    { path: "/:pathMatch(.*)*", redirect: "/spaces" },
  ],
});
router.beforeEach(async (to, from) => {
  if (from.name !== "spaces" || to.name === "spaces") return true;
  try {
    await useSpaceStore().commitWorkspaceNavigation();
    return true;
  } catch (error) {
    if (!isApplicationLeaveCancelled(error)) {
      void logError(`Space commit prevented navigation: ${formatErrorDiagnostic(error)}`);
      toast.error(getErrorMessage(error));
    }
    return false;
  }
});
