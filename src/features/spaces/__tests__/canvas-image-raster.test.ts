import { mount } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";
import { defineComponent, readonly } from "vue";

import { defaultImageEdit } from "../image-editing/image-edit";
import type { ImageRenderRequest } from "../image-editing/image-render-worker";

const rendering = vi.hoisted(() => ({
  render:
    vi.fn<
      (
        owner: object,
        prepare: () => Promise<ImageRenderRequest>,
        priority: boolean,
      ) => Promise<ImageBitmap | undefined>
    >(),
  cancel: vi.fn<(owner: object) => void>(),
}));
vi.mock("../image-editing/use-image-render-worker", () => ({ useImageRenderWorker: () => rendering }));
import { useCanvasImageRaster } from "../presentation/canvas/nodes/use-canvas-image-raster";

afterEach(() => vi.unstubAllGlobals());
it("captures a transferable edit snapshot even when a crop view contains nested readonly proxies", async () => {
  const bitmap = { close: vi.fn<() => void>() } as unknown as ImageBitmap;
  vi.stubGlobal("createImageBitmap", vi.fn<typeof createImageBitmap>().mockResolvedValue(bitmap));
  const active = readonly(defaultImageEdit());
  const cropPreview = { ...active, crop: { x: 0, y: 0, width: 1, height: 1 } };
  let captured: ImageRenderRequest | undefined;
  rendering.render.mockImplementation(async (_owner, prepare) => {
    captured = await prepare();
    return undefined;
  });
  let raster!: ReturnType<typeof useCanvasImageRaster>;
  const wrapper = mount(
    defineComponent({
      setup() {
        raster = useCanvasImageRaster({
          image: new Image(),
          size: () => ({ width: 100, height: 100, pixelRatio: 2 }),
          edit: () => cropPreview,
          ready: () => true,
          active: () => false,
          paint: () => undefined,
          error: () => undefined,
        });
        return () => null;
      },
    }),
  );
  try {
    expect(() => raster.update()).not.toThrow();
    await vi.waitFor(() => expect(captured).toBeDefined());
    expect(() => structuredClone(captured!.edit)).not.toThrow();
    expect(captured!.edit).toEqual(defaultImageEdit());
    expect(captured!.edit.adjustments).not.toBe(cropPreview.adjustments);
  } finally {
    wrapper.unmount();
  }
});
