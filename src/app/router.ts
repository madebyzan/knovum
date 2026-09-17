import { createRouter, createWebHashHistory } from "vue-router";

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
    },
    { path: "/:pathMatch(.*)*", redirect: "/spaces" },
  ],
});
