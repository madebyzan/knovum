import { ScanLine } from "@lucide/vue";
import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { DropdownMenuContent, DropdownMenuRoot, ToolbarRoot } from "reka-ui";
import { beforeEach, describe, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h } from "vue";

import { i18n } from "../../../../../../foundation/i18n/instance";
import { availableWhen } from "../../../../../../shared/contracts/action-availability";
import { localModelAvailability } from "../../../../__tests__/support/image-operation-fixtures";
import type { ImageExecutionTarget } from "../../../../contracts/generated/ImageExecutionTarget";
import type { ImageOperationCapabilities } from "../../../../contracts/generated/ImageOperationCapabilities";
import SpaceImageDirectAction from "../SpaceImageDirectAction.vue";

const mocks = vi.hoisted(() => ({
  capabilities: vi.fn<() => Promise<ImageOperationCapabilities>>(),
  catalog: vi.fn<() => Promise<unknown[]>>(),
}));

vi.mock("../../../../knowledge-derivations/image-operation", () => ({
  getImageOperationCapabilities: mocks.capabilities,
}));
vi.mock("../../../../../../foundation/ipc/invoke-command", () => ({
  invokeCommand: (cmd: string) =>
    cmd === "plugin:local-resources|get_model_catalog" ? mocks.catalog() : Promise.resolve(undefined),
}));

const LITE: ImageExecutionTarget = { type: "local", model: "birefnetLite", specId: "birefnetLite-spec" };
const ANIME: ImageExecutionTarget = { type: "local", model: "isnetAnime", specId: "isnetAnime-spec" };

function setTargets(models: { id: "birefnetLite" | "isnetAnime"; selected?: boolean }[]) {
  mocks.capabilities.mockResolvedValue({
    targets: models.map((m) => ({
      capability: "imageSegmentation",
      label: m.id,
      selected: Boolean(m.selected),
      execution: { type: "local", model: m.id, specId: `${m.id}-spec` },
      availability: { state: "ready" },
      maximumSide: null,
      maximumPixels: null,
    })),
    remote: null,
    selection: { state: "ready" },
    depth: { state: "ready" },
    upscale: { state: "ready" },
    multiAngle: { state: "ready" },
    maskedRemoval: { state: "ready" },
    promptRemoval: { state: "ready" },
  });
  mocks.catalog.mockResolvedValue(
    models.map((m) =>
      localModelAvailability(m.id, "imageSegmentation", { selected: Boolean(m.selected), verified: true }),
    ),
  );
}

function mountToolbar(onRun: (execution?: ImageExecutionTarget) => void) {
  const Wrapper = defineComponent({
    setup() {
      return () =>
        h(ToolbarRoot, null, () =>
          h(SpaceImageDirectAction, {
            capability: "imageSegmentation",
            availability: availableWhen(true, "image_preparing"),
            reason: "",
            icon: ScanLine,
            labelKey: "spaces.creation.image.removeBackground",
            onRun,
          }),
        );
    },
  });
  const wrapper = mount(Wrapper, {
    attachTo: document.body,
    global: {
      plugins: [
        i18n,
        [VueQueryPlugin, { queryClient: new QueryClient({ defaultOptions: { queries: { retry: false } } }) }],
      ],
    },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

beforeEach(() => vi.clearAllMocks());

describe("SpaceImageDirectAction", () => {
  it("runs immediately on click when only one model is downloaded", async () => {
    setTargets([{ id: "birefnetLite", selected: true }]);
    const onRun = vi.fn<(execution?: ImageExecutionTarget) => void>();
    const wrapper = mountToolbar(onRun);
    await flushPromises();
    await wrapper.get("button").trigger("click");
    expect(onRun).toHaveBeenCalledWith(LITE);
  });

  it("opens a lightweight popover when multiple models are downloaded and runs on single click", async () => {
    setTargets([{ id: "birefnetLite", selected: true }, { id: "isnetAnime" }]);
    const onRun = vi.fn<(execution?: ImageExecutionTarget) => void>();
    const wrapper = mountToolbar(onRun);
    await flushPromises();

    await wrapper.get("button").trigger("click");
    await flushPromises();
    expect(onRun).not.toHaveBeenCalled();

    const choices = Array.from(document.body.querySelectorAll<HTMLButtonElement>("[role='dialog'] button"));
    expect(choices.map((b) => b.textContent)).toEqual([
      expect.stringContaining("birefnetLite"),
      expect.stringContaining("isnetAnime"),
    ]);
    choices[1]!.click();
    await flushPromises();
    expect(onRun).toHaveBeenCalledWith(ANIME);
    expect(document.body.querySelector("[role='dialog']")).toBeNull();
  });

  it("renders a direct submenu item in menuItem mode when multiple models exist", async () => {
    setTargets([{ id: "birefnetLite", selected: true }, { id: "isnetAnime" }]);
    const onRun = vi.fn<(execution?: ImageExecutionTarget) => void>();
    const MenuWrapper = defineComponent({
      setup() {
        return () =>
          h(DropdownMenuRoot, { open: true }, () =>
            h(DropdownMenuContent, null, () =>
              h(SpaceImageDirectAction, {
                capability: "imageSegmentation",
                availability: availableWhen(true, "image_preparing"),
                reason: "",
                icon: ScanLine,
                labelKey: "spaces.creation.image.removeBackground",
                menuItem: true,
                onRun,
              }),
            ),
          );
      },
    });
    const wrapper = mount(MenuWrapper, {
      attachTo: document.body,
      global: {
        plugins: [
          i18n,
          [
            VueQueryPlugin,
            { queryClient: new QueryClient({ defaultOptions: { queries: { retry: false } } }) },
          ],
        ],
      },
    });
    onTestFinished(() => wrapper.unmount());
    await flushPromises();
    expect(document.body.querySelector("[role='menuitem'][aria-haspopup='menu']")).not.toBeNull();
  });
});
