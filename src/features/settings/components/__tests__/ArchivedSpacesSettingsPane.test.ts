import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { shallowRef } from "vue";

import {
  archiveSettingsKey,
  type ArchiveSettingsContract,
} from "../../../../shared/contracts/archive-settings";
import ArchivedSpacesSettingsPane from "../panes/ArchivedSpacesSettingsPane.vue";

const toast = vi.hoisted(() => ({ error: vi.fn<(message: string, options?: object) => void>() }));
vi.mock("vue-sonner", () => ({ toast }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

function mountPane(contract: ArchiveSettingsContract) {
  return mount(ArchivedSpacesSettingsPane, {
    global: { provide: { [archiveSettingsKey as symbol]: contract } },
  });
}

describe("Archived Spaces settings", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows initialization failure and retries", async () => {
    const initialize = vi
      .fn<(title: string) => Promise<void>>()
      .mockRejectedValueOnce(new Error("database unavailable"))
      .mockResolvedValueOnce(undefined);
    const wrapper = mountPane({
      spaces: shallowRef([]),
      initialize,
      restore: vi.fn<(spaceId: string) => Promise<void>>(),
      permanentlyDelete: vi.fn<(spaceId: string) => Promise<void>>(),
      permanentlyDeleteAll: vi.fn<() => Promise<void>>(),
    });
    await flushPromises();

    expect(wrapper.get('[data-testid="archive-load-error"]').text()).toContain("database unavailable");
    await wrapper.get('[data-testid="archive-retry"]').trigger("click");
    await flushPromises();

    expect(initialize).toHaveBeenCalledTimes(2);
    expect(wrapper.find('[data-testid="archive-load-error"]').exists()).toBe(false);
    wrapper.unmount();
  });

  it("prevents duplicate restore and reports rejection", async () => {
    let rejectRestore!: (reason: unknown) => void;
    const restore = vi.fn<(spaceId: string) => Promise<void>>(
      () => new Promise<void>((_resolve, reject) => (rejectRestore = reject)),
    );
    const wrapper = mountPane({
      spaces: shallowRef([{ id: "space", title: "Research" }]),
      initialize: vi.fn<(title: string) => Promise<void>>().mockResolvedValue(undefined),
      restore,
      permanentlyDelete: vi.fn<(spaceId: string) => Promise<void>>(),
      permanentlyDeleteAll: vi.fn<() => Promise<void>>(),
    });
    await flushPromises();
    const button = wrapper.get('[data-testid="archive-restore-space"]');
    await button.trigger("click");
    await button.trigger("click");
    expect(restore).toHaveBeenCalledOnce();

    rejectRestore(new Error("restore rejected"));
    await flushPromises();
    expect(toast.error).toHaveBeenCalledWith("settings.general.archived.restoreFailed", {
      description: "restore rejected",
    });
    expect(button.attributes("disabled")).toBeUndefined();
    wrapper.unmount();
  });
});
