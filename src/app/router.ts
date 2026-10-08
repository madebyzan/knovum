import { pick } from "es-toolkit";
import { defineAsyncComponent } from "vue";
import { createRouter, createWebHashHistory } from "vue-router";

import { listSpaceActions } from "../features/spaces/workflows/action-catalog";

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
        voiceLibraryComponent: defineAsyncComponent(
          () => import("../features/spaces/presentation/speech/SpaceVoiceLibrarySettings.vue"),
        ),
        extensionsComponent: defineAsyncComponent(
          () => import("../features/extensions/AgentExtensionsSettings.vue"),
        ),
        loadGenerationActions: async () =>
          (await listSpaceActions()).map((action) => pick(action, ["id", "routingPurpose"])),
      }),
    },
    { path: "/:pathMatch(.*)*", redirect: "/spaces" },
  ],
});
