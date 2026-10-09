import { error as logError } from "@tauri-apps/plugin-log";
import { pick } from "es-toolkit";
import { defineAsyncComponent } from "vue";
import { createRouter, createWebHashHistory } from "vue-router";
import { toast } from "vue-sonner";

import { useSpaceStore } from "../features/spaces/session/space-store";
import { listSpaceActions } from "../features/spaces/workflows/action-catalog";
import { formatErrorDiagnostic, getErrorMessage } from "../foundation/errors/error-format";
import { isApplicationLeaveCancelled } from "../shared/contracts/application-leave";
import { returnToWorkspace } from "./workspace-session";

const voiceLibraryComponent = defineAsyncComponent(
  () => import("../features/spaces/presentation/speech/VoiceLibrarySettings.vue"),
);
const extensionsComponent = defineAsyncComponent(
  () => import("../features/extensions/AgentExtensionsSettings.vue"),
);

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/spaces" },
    {
      path: "/spaces",
      name: "spaces",
      component: () => import("../features/spaces/SpaceView.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("../features/settings/SettingsView.vue"),
      props: () => ({
        voiceLibraryComponent,
        extensionsComponent,
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
    await useSpaceStore().commitActiveSession();
    return true;
  } catch (error) {
    if (!isApplicationLeaveCancelled(error)) {
      void logError(`Space commit prevented navigation: ${formatErrorDiagnostic(error)}`);
      toast.error(getErrorMessage(error));
    }
    return false;
  }
});
