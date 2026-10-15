import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { nextTick } from "vue";

import CanvasGridSurface from "../konva/rendering/CanvasGridSurface.vue";
import type { CanvasPalette } from "../konva/rendering/use-canvas-palette";
import { createCanvasCamera } from "../konva/viewport/canvas-camera";

const palette = { grid: "#abcdef" } as unknown as CanvasPalette;

async function mountGrid(zoom: number) {
  const camera = createCanvasCamera({ x: 0, y: 0, zoom });
  const wrapper = mount(CanvasGridSurface, { props: { camera, palette, zoom } });
  await nextTick();
  return { camera, dot: wrapper.get("circle").element, pattern: wrapper.get("pattern").element, wrapper };
}

describe("canvas grid surface", () => {
  it("keeps dots on world multiples of the spacing as the camera moves", async () => {
    const { camera, pattern, wrapper } = await mountGrid(1);
    expect(pattern.getAttribute("patternTransform")).toBe("translate(12.5 12.5)");

    camera.set({ x: 30, y: -5, zoom: 1 });
    camera.settle();

    // 30 - 12.5 leaves 17.5 inside a cell of 25; -5 - 12.5 is negative and wraps to 7.5.
    expect(pattern.getAttribute("patternTransform")).toBe("translate(17.5 7.5)");
    wrapper.unmount();
  });

  it("scales the cell with the zoom while the dot stays the same size on screen", async () => {
    const { dot, pattern, wrapper } = await mountGrid(1);
    expect(pattern.getAttribute("width")).toBe("25");
    const radius = dot.getAttribute("r");

    await wrapper.setProps({ camera: createCanvasCamera({ x: 0, y: 0, zoom: 2 }), zoom: 2 });

    expect(pattern.getAttribute("width")).toBe("50");
    expect(pattern.getAttribute("height")).toBe("50");
    expect(dot.getAttribute("cx")).toBe("25");
    expect(dot.getAttribute("r")).toBe(radius);
    wrapper.unmount();
  });
});
