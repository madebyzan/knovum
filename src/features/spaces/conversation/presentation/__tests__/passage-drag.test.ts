import { afterEach, describe, expect, it, vi } from "vitest";
import { effectScope } from "vue";

import { usePassageDrag } from "../use-passage-drag";

type Point = { x: number; y: number };
const PASSAGE = { text: "Words", citations: [1] };

function setup(onPassage = true) {
  const options = {
    pressed: vi.fn<(point: Point) => typeof PASSAGE | undefined>(() => (onPassage ? PASSAGE : undefined)),
    canvasPointAt: vi.fn<(point: Point) => Point | undefined>((point) =>
      point.x > 500 ? { x: point.x - 500, y: point.y } : undefined,
    ),
    drop: vi.fn<(passage: typeof PASSAGE, point: Point) => Promise<void>>(async () => {}),
    click: vi.fn<() => void>(),
  };
  const scope = effectScope();
  const drag = scope.run(() => usePassageDrag(options))!;
  scopes.push(scope);
  return { options, drag };
}

const scopes: ReturnType<typeof effectScope>[] = [];
afterEach(() => {
  for (const scope of scopes.splice(0)) scope.stop();
});

function press(drag: ReturnType<typeof setup>["drag"], x = 10, y = 10) {
  const event = new MouseEvent("mousedown", { button: 0, clientX: x, clientY: y, cancelable: true });
  drag.start(event);
  return event;
}
const mouse = (type: "mousemove" | "mouseup", x: number, y: number) =>
  globalThis.dispatchEvent(new MouseEvent(type, { clientX: x, clientY: y }));

describe("dragging a passage onto the canvas", () => {
  it("leaves presses off the selected words to the browser", () => {
    const { drag, options } = setup(false);
    expect(press(drag).defaultPrevented).toBe(false);
    mouse("mousemove", 700, 10);
    mouse("mouseup", 700, 10);
    expect(drag.drag.value).toBeUndefined();
    expect(options.drop).not.toHaveBeenCalled();
  });

  it("treats a press that does not move as a click", () => {
    const { drag, options } = setup();
    expect(press(drag).defaultPrevented).toBe(true);
    mouse("mousemove", 12, 12);
    expect(drag.drag.value).toBeUndefined();
    mouse("mouseup", 12, 12);
    expect(options.click).toHaveBeenCalledOnce();
    expect(options.drop).not.toHaveBeenCalled();
  });

  it("follows the pointer, knows when it is over the canvas, and drops there", () => {
    const { drag, options } = setup();
    press(drag);
    mouse("mousemove", 200, 40);
    expect(drag.drag.value).toEqual({ passage: PASSAGE, x: 200, y: 40, target: undefined });
    mouse("mousemove", 640, 80);
    expect(drag.drag.value?.target).toEqual({ x: 140, y: 80 });
    mouse("mouseup", 650, 90);
    expect(drag.drag.value).toBeUndefined();
    expect(options.drop).toHaveBeenCalledExactlyOnceWith(PASSAGE, { x: 150, y: 90 });
    expect(options.click).not.toHaveBeenCalled();
  });

  it("drops nothing when released off the canvas or cancelled with Escape", () => {
    const { drag, options } = setup();
    press(drag);
    mouse("mousemove", 200, 40);
    mouse("mouseup", 200, 40);
    press(drag);
    mouse("mousemove", 640, 80);
    globalThis.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(drag.drag.value).toBeUndefined();
    mouse("mouseup", 640, 80);
    expect(options.drop).not.toHaveBeenCalled();
  });
});
