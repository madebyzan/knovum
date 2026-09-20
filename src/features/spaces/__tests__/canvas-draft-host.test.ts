import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import type { CanvasHostEvents } from "../canvas/contract/canvas-editor";
import { flushSpaceSessionPersistence } from "../canvas/editor-session";
import { emptyCanvasDocument } from "../testing/canvas-fixtures";

const repository = vi.hoisted(() => ({
  load: vi.fn(async () => null),
  save: vi.fn(async () => 1),
}));
vi.mock("../canvas/persistence/canvas-state-repository", () => ({ canvasStateRepository: repository }));
vi.mock("../canvas/konva/KonvaCanvasSurface.vue", () => ({
  default: {
    name: "KonvaCanvasSurface",
    methods: { flushViewport() {} },
    props: ["document", "events"],
    emits: ["documentChange"],
    template: "<div />",
  },
}));

import CanvasSurfaceHost from "../canvas/host/CanvasSurfaceHost.vue";

describe("draft canvas host", () => {
  it("saves the draft viewport after materialization without remounting", async () => {
    const onError = vi.fn();
    const wrapper = mount(CanvasSurfaceHost, {
      props: {
        draft: true,
        // The stub surface never invokes editor events; only persistence errors are observed.
        events: { onError } as CanvasHostEvents,
        importJsonCanvas: async () => {
          throw new Error("Import is not used by this test");
        },
        focusNodeIds: [],
        interactionEnabled: true,
        tool: "select",
        nodes: [],
        relations: [],
        spaceId: "draft-host",
      },
    });
    try {
      await flushPromises();
      const surface = wrapper.findComponent({ name: "KonvaCanvasSurface" });
      const viewport = { ...emptyCanvasDocument, viewport: { x: 12, y: 34, zoom: 2 } };
      surface.vm.$emit("documentChange", viewport);
      await flushSpaceSessionPersistence();
      expect(repository.save).not.toHaveBeenCalled();

      await wrapper.setProps({ draft: false });
      await flushSpaceSessionPersistence();

      expect(repository.save).toHaveBeenCalledOnce();
      expect(repository.save).toHaveBeenCalledWith({
        spaceId: "draft-host",
        baseRevision: 0,
        document: viewport,
      });
      expect(repository.load).toHaveBeenCalledOnce();
      expect(wrapper.findComponent({ name: "KonvaCanvasSurface" }).vm).toBe(surface.vm);
      expect(onError).not.toHaveBeenCalled();
    } finally {
      wrapper.unmount();
      await flushPromises();
      vi.clearAllMocks();
    }
  });
});
