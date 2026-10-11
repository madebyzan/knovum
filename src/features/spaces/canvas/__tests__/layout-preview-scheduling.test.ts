import { mount } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";
import { defineComponent } from "vue";

import type { CanvasGestureUpdate } from "../konva/interaction/canvas-gesture-update";
import { useLayoutPreview } from "../konva/interaction/use-layout-preview";

const update = (x: number): CanvasGestureUpdate[] => [
  { nodeId: "text", frame: { x, y: 0, width: 100, height: 40, rotation: 0 } },
];
afterEach(() => vi.unstubAllGlobals());

it("publishes the latest layout once per frame and cancels cleared work", () => {
  const scheduled = new Map<number, FrameRequestCallback>();
  let nextId = 0;
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    scheduled.set(++nextId, callback);
    return nextId;
  });
  vi.stubGlobal("cancelAnimationFrame", (id: number) => scheduled.delete(id));
  let preview!: ReturnType<typeof useLayoutPreview>;
  const wrapper = mount(
    defineComponent({
      setup() {
        preview = useLayoutPreview();
        return () => null;
      },
    }),
  );
  try {
    preview.update(update(10));
    preview.update(update(20));
    expect(preview.gestureUpdateByNodeId.value.size).toBe(0);
    expect(scheduled.size).toBe(1);
    const [id, frame] = [...scheduled][0]!;
    scheduled.delete(id);
    frame(0);
    expect(preview.gestureUpdateByNodeId.value.get("text")?.frame.x).toBe(20);
    preview.update(update(30));
    preview.update([]);
    expect(scheduled.size).toBe(0);
    expect(preview.gestureUpdateByNodeId.value.size).toBe(0);
    preview.update(update(40));
    wrapper.unmount();
    expect(scheduled.size).toBe(0);
  } finally {
    if (wrapper.exists()) wrapper.unmount();
  }
});
