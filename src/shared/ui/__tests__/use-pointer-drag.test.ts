import { afterEach, describe, expect, it, vi } from "vitest";
import { effectScope } from "vue";

import { isTopmostAt, usePointerDrag, type ViewportPoint } from "../use-pointer-drag";

const ITEM = { id: "item" };
const scopes: ReturnType<typeof effectScope>[] = [];

function setup() {
  const options = {
    targetAt: vi.fn<(point: ViewportPoint) => number | undefined>((point) =>
      point.x > 500 ? point.x - 500 : undefined,
    ),
    drop: vi.fn<(item: typeof ITEM, target: number) => void>(),
    tap: vi.fn<() => void>(),
  };
  const scope = effectScope();
  scopes.push(scope);
  return { options, drag: scope.run(() => usePointerDrag(options))! };
}

function press(drag: ReturnType<typeof setup>["drag"], button = 0) {
  const event = new MouseEvent("mousedown", { button, clientX: 10, clientY: 10, cancelable: true });
  drag.start(event, ITEM);
  return event;
}
const mouse = (type: "mousemove" | "mouseup" | "click", x: number) =>
  globalThis.dispatchEvent(new MouseEvent(type, { clientX: x, clientY: 10, cancelable: true }));

afterEach(() => {
  for (const scope of scopes.splice(0)) scope.stop();
});

describe("dragging with the mouse", () => {
  it("takes the press from the browser, and leaves other buttons to it", () => {
    const { drag, options } = setup();
    expect(press(drag, 2).defaultPrevented).toBe(false);
    mouse("mousemove", 700);
    expect(drag.drag.value).toBeUndefined();
    expect(press(drag).defaultPrevented).toBe(true);
    expect(options.drop).not.toHaveBeenCalled();
  });

  it("treats a press that does not move past the threshold as a tap, and lets its click through", () => {
    const { drag, options } = setup();
    press(drag);
    mouse("mousemove", 13);
    expect(drag.drag.value).toBeUndefined();
    mouse("mouseup", 13);
    expect(options.tap).toHaveBeenCalledOnce();
    const click = new MouseEvent("click", { cancelable: true });
    globalThis.dispatchEvent(click);
    expect(click.defaultPrevented).toBe(false);
  });

  it("follows the pointer, knows where it would land, and drops there without a click", () => {
    const { drag, options } = setup();
    press(drag);
    mouse("mousemove", 200);
    expect(drag.drag.value).toEqual({ item: ITEM, x: 200, y: 10, target: undefined });
    mouse("mousemove", 640);
    expect(drag.drag.value?.target).toBe(140);
    mouse("mouseup", 650);
    expect(drag.drag.value).toBeUndefined();
    expect(options.drop).toHaveBeenCalledExactlyOnceWith(ITEM, 150);
    const click = new MouseEvent("click", { cancelable: true });
    globalThis.dispatchEvent(click);
    expect(click.defaultPrevented).toBe(true);
    expect(options.tap).not.toHaveBeenCalled();
  });

  it("drops nothing when released elsewhere, cancelled with Escape or the window loses focus", () => {
    const { drag, options } = setup();
    for (const cancel of [
      () => mouse("mouseup", 200),
      () => globalThis.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" })),
      () => globalThis.dispatchEvent(new Event("blur")),
    ]) {
      press(drag);
      mouse("mousemove", 640);
      cancel();
      expect(drag.drag.value).toBeUndefined();
      mouse("mouseup", 640);
    }
    expect(options.drop).not.toHaveBeenCalled();
  });
});

describe("what the pointer is over", () => {
  it("is the element only where nothing is stacked above it", () => {
    const target = document.createElement("div");
    const child = target.appendChild(document.createElement("span"));
    const cover = document.createElement("div");
    document.body.append(target, cover);
    const at = vi.spyOn(document, "elementFromPoint");
    at.mockReturnValueOnce(child);
    expect(isTopmostAt(target, { x: 1, y: 1 })).toBe(true);
    at.mockReturnValueOnce(cover);
    expect(isTopmostAt(target, { x: 1, y: 1 })).toBe(false);
    expect(isTopmostAt(null, { x: 1, y: 1 })).toBe(false);
    at.mockRestore();
    document.body.replaceChildren();
  });
});
