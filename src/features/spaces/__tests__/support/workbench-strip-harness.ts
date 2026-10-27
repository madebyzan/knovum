import { mount } from "@vue/test-utils";
import { onTestFinished } from "vitest";

import { i18n } from "../../../../foundation/i18n/instance";
import { workspaceRightSidebarKey } from "../../../../shared/contracts/workspace-right-sidebar";
import { useSpaceStore } from "../../session/space-store";
import SpaceWorkbenchTabs from "../../workspace/presentation/SpaceWorkbenchTabs.vue";

/** Opens the Inbox with document tabs for nodes "a" and "b" of its projection. */
export async function openWorkbenchWithDocuments() {
  const store = useSpaceStore();
  await store.initialize("Inbox");
  await store.openDocumentEditor("a");
  await store.openDocumentEditor("b");
  return store;
}

export function mountWorkbenchStrip() {
  const wrapper = mount(SpaceWorkbenchTabs, {
    attachTo: document.body,
    global: {
      plugins: [i18n],
      provide: { [workspaceRightSidebarKey as symbol]: { open: () => {}, close: () => {} } },
    },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}
