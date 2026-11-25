import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import type { UpdateState } from "../../../shared/contracts/generated/UpdateState";
import WorkspaceUpdateIndicator from "../WorkspaceUpdateIndicator.vue";

const storeState = vi.hoisted(() => ({
  state: null as UpdateState | null,
  requestPending: false,
  installing: false,
  blockedByTasks: false,
  canInstall: true,
  download: vi.fn<() => Promise<void>>(),
  cancelDownload: vi.fn<() => Promise<void>>(),
  install: vi.fn<() => Promise<void>>(),
}));

const reactiveState = shallowRef<UpdateState | null>(null);
const reactiveBlocked = shallowRef(false);
const reactiveCanInstall = shallowRef(true);

vi.mock("../../../foundation/updates/update-store", () => ({
  useUpdateStore: () => ({
    get state() {
      return reactiveState.value;
    },
    get requestPending() {
      return storeState.requestPending;
    },
    get installing() {
      return storeState.installing;
    },
    get blockedByTasks() {
      return reactiveBlocked.value;
    },
    get canInstall() {
      return reactiveCanInstall.value;
    },
    download: storeState.download,
    cancelDownload: storeState.cancelDownload,
    install: storeState.install,
  }),
}));

const release = { version: "0.4.1", currentVersion: "0.4.0", notes: "# Knovum 0.4.1\n\nCleaner updates." };

function mountIndicator() {
  return mount(WorkspaceUpdateIndicator, {
    global: {
      plugins: [createPinia(), i18n],
      stubs: {
        UiPopover: {
          props: ["open"],
          template: '<div><slot name="trigger" /><div data-testid="popover-body"><slot /></div></div>',
        },
        UiScrollArea: { template: "<div><slot /></div>" },
        UiMarkdown: { props: ["content"], template: '<div data-testid="md">{{ content }}</div>' },
      },
    },
  });
}

describe("WorkspaceUpdateIndicator", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    reactiveState.value = null;
    reactiveBlocked.value = false;
    reactiveCanInstall.value = true;
    storeState.download.mockResolvedValue(undefined);
    storeState.cancelDownload.mockResolvedValue(undefined);
    storeState.install.mockResolvedValue(undefined);
  });

  it("stays hidden when no actionable update is active", () => {
    reactiveState.value = { status: "current" };
    const wrapper = mountIndicator();
    expect(wrapper.find('[data-testid="workspace-update-indicator"]').exists()).toBe(false);
    wrapper.unmount();
  });

  it("shows release notes and triggers download directly from the toolbar popover", async () => {
    reactiveState.value = { status: "available", release };
    const wrapper = mountIndicator();

    expect(wrapper.find('[data-testid="workspace-update-indicator"]').exists()).toBe(true);
    expect(wrapper.get('[data-testid="md"]').text()).toBe("Cleaner updates.");

    await wrapper.get('[data-testid="toolbar-update-download"]').trigger("click");
    expect(storeState.download).toHaveBeenCalledOnce();

    await wrapper.get('[data-testid="toolbar-update-settings"]').trigger("click");
    expect(wrapper.emitted("openSettings")).toHaveLength(1);
    wrapper.unmount();
  });

  it("shows live download progress and supports cancellation", async () => {
    reactiveState.value = {
      status: "downloading",
      release,
      downloaded: 25 * 1024 * 1024,
      total: 100 * 1024 * 1024,
    };
    const wrapper = mountIndicator();

    expect(wrapper.get('[data-testid="workspace-update-indicator"]').text()).toContain("25%");
    expect(wrapper.find('[data-testid="toolbar-update-progress"]').exists()).toBe(true);

    await wrapper.get('[data-testid="toolbar-update-cancel"]').trigger("click");
    expect(storeState.cancelDownload).toHaveBeenCalledOnce();
    wrapper.unmount();
  });

  it("switches to restart-and-install once verified and respects task blocking", async () => {
    reactiveState.value = { status: "ready", release };
    reactiveBlocked.value = true;
    reactiveCanInstall.value = false;
    const wrapper = mountIndicator();

    const installButton = wrapper.get('[data-testid="toolbar-update-install"]');
    expect(installButton.attributes("disabled")).toBeDefined();

    reactiveBlocked.value = false;
    reactiveCanInstall.value = true;
    await nextTick();
    await installButton.trigger("click");
    expect(storeState.install).toHaveBeenCalledOnce();
    wrapper.unmount();
  });
});
