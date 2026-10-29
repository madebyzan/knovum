import { enableAutoUnmount, mount } from "@vue/test-utils";
import { afterEach, describe, expect, it, vi } from "vitest";
import { shallowRef } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { VideoMaterialNode } from "../../material";
import SpaceVideoMaterialBrowser from "../SpaceVideoMaterialBrowser.vue";
import type { VideoEditor } from "../use-video-editor";

const clip = {
  id: "clip",
  type: "audio",
  data: {
    kind: "audio",
    title: "Theme",
    assetPath: "theme.mp3",
    fileName: "theme.mp3",
    sourceRevision: "r1",
  },
  initialPosition: { x: 0, y: 0 },
} as unknown as VideoMaterialNode;

function mountBrowser() {
  const materials = {
    nodes: shallowRef([clip]),
    busy: shallowRef(false),
    add: vi.fn<(node: VideoMaterialNode, atMs: number) => Promise<void>>(async () => {}),
    importFiles: vi.fn<() => void>(),
  };
  // The timeline sits right of x = 500 and a second there is 100 px.
  const timeAtPoint = vi.fn<(point: { x: number }) => number | undefined>((point) =>
    point.x > 500 ? (point.x - 500) * 10 : undefined,
  );
  const editor = { materials, player: { positionMs: () => 2_500 }, timeline: { timeAtPoint } };
  const wrapper = mount(SpaceVideoMaterialBrowser, {
    attachTo: document.body,
    props: { editor: editor as unknown as VideoEditor },
    global: { plugins: [i18n], stubs: { UiScrollArea: { template: "<div><slot /></div>" } } },
  });
  return { wrapper, materials };
}

const material = (wrapper: ReturnType<typeof mountBrowser>["wrapper"]) =>
  wrapper.get(
    `[aria-label="${i18n.global.t("spaces.media.videoEditor.materials.add", { title: "Theme" })}"]`,
  );
const mouse = (type: "mousemove" | "mouseup", x: number) =>
  globalThis.dispatchEvent(new MouseEvent(type, { clientX: x, clientY: 10 }));

enableAutoUnmount(afterEach);

describe("adding a material to the timeline", () => {
  it("adds a clicked material at the playhead", async () => {
    const { wrapper, materials } = mountBrowser();
    await material(wrapper).trigger("mousedown", { button: 0, clientX: 10, clientY: 10 });
    mouse("mouseup", 10);
    await material(wrapper).trigger("click");
    expect(materials.add).toHaveBeenCalledExactlyOnceWith(clip, 2_500);
  });

  it("adds a material dragged onto the timeline where it is dropped, without also clicking it", async () => {
    const { wrapper, materials } = mountBrowser();
    await material(wrapper).trigger("mousedown", { button: 0, clientX: 10, clientY: 10 });
    mouse("mousemove", 300);
    await wrapper.vm.$nextTick();
    expect(document.body.textContent).toContain("Theme");
    mouse("mousemove", 800);
    mouse("mouseup", 800);
    await material(wrapper).trigger("click");
    expect(materials.add).toHaveBeenCalledExactlyOnceWith(clip, 3_000);
  });

  it("adds nothing when the material is dropped off the timeline", async () => {
    const { wrapper, materials } = mountBrowser();
    await material(wrapper).trigger("mousedown", { button: 0, clientX: 10, clientY: 10 });
    mouse("mousemove", 300);
    mouse("mouseup", 300);
    expect(materials.add).not.toHaveBeenCalled();
  });
});
