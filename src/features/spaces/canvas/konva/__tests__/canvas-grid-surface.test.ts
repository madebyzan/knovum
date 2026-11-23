import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { nextTick } from "vue";

import type { CanvasPalette } from "../../use-canvas-palette";
import CanvasBackdropSurface from "../rendering/CanvasBackdropSurface.vue";
import CanvasGridSurface from "../rendering/CanvasGridSurface.vue";
import { createCanvasCamera } from "../viewport/canvas-camera";

const palette = { grid: "#abcdef", gridLine: "#123456" } as unknown as CanvasPalette;
const surface = { width: 1536, height: 1024 };

async function mountGrid(zoom: number, patternMode: "dots" | "grid" = "dots") {
  const camera = createCanvasCamera({ x: 0, y: 0, zoom });
  const wrapper = mount(CanvasGridSurface, {
    props: { camera, palette, pattern: patternMode, surface, zoom },
  });
  await nextTick();
  const dotWrapper = wrapper.find("circle");
  const gridWrapper = wrapper.find("path");
  return {
    camera,
    dot: dotWrapper.exists() ? (dotWrapper.element as SVGCircleElement) : undefined,
    gridLines: gridWrapper.exists() ? (gridWrapper.element as SVGPathElement) : undefined,
    pattern: wrapper.get("pattern").element,
    wrapper,
  };
}

describe("canvas grid surface", () => {
  it("spans the stage surface on its own layer, so a moving host edge only clips it", async () => {
    const { wrapper } = await mountGrid(1);
    const svg = wrapper.get("svg");

    expect(svg.attributes("width")).toBe("1536");
    expect(svg.attributes("height")).toBe("1024");
    expect(svg.classes()).toContain("will-change-transform");
    expect(svg.classes()).not.toContain("inset-0");
    wrapper.unmount();
  });

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
    const radius = dot?.getAttribute("r");

    await wrapper.setProps({ camera: createCanvasCamera({ x: 0, y: 0, zoom: 2 }), zoom: 2 });

    expect(pattern.getAttribute("width")).toBe("50");
    expect(pattern.getAttribute("height")).toBe("50");
    expect(dot?.getAttribute("cx")).toBe("25");
    expect(dot?.getAttribute("r")).toBe(radius);
    wrapper.unmount();
  });

  it("renders world-aligned grid lines that scale cell geometry with zoom", async () => {
    const { camera, gridLines, pattern, wrapper } = await mountGrid(1, "grid");
    expect(gridLines?.getAttribute("stroke")).toBe("#123456");
    expect(gridLines?.getAttribute("d")).toBe("M 25 0 L 0 0 0 25");
    expect(pattern.getAttribute("patternTransform")).toBe("translate(0 0)");

    camera.set({ x: 30, y: -5, zoom: 1 });
    camera.settle();
    expect(pattern.getAttribute("patternTransform")).toBe("translate(5 20)");

    await wrapper.setProps({ camera: createCanvasCamera({ x: 0, y: 0, zoom: 2 }), zoom: 2 });
    expect(pattern.getAttribute("width")).toBe("50");
    expect(gridLines?.getAttribute("d")).toBe("M 50 0 L 0 0 0 50");
    wrapper.unmount();
  });

  it("detaches its camera frame observer when unmounted", async () => {
    const { camera, pattern, wrapper } = await mountGrid(1);
    wrapper.unmount();

    camera.set({ x: 40, y: 40, zoom: 1 });
    camera.settle();
    expect(pattern.getAttribute("patternTransform")).toBe("translate(12.5 12.5)");
  });

  it("renders the viewport-anchored curated backdrop and contrast veil", () => {
    const wrapper = mount(CanvasBackdropSurface, {
      props: { backdrop: "summer-glow" },
    });

    expect(wrapper.attributes("aria-hidden")).toBe("true");
    expect(wrapper.get("img").attributes("src")).toBeTruthy();
    expect(wrapper.html()).toContain("var(--knovum-space-backdrop-veil)");
    wrapper.unmount();
  });
});
