import { describe, expect, it } from "vitest";

import { useSidebarResize } from "../use-sidebar-resize";

const geometry = { minSize: 200, maxSize: 320 };

function createSash(edge: "leading" | "trailing", startWidth = 240) {
  const state = { commits: 0, width: startWidth };
  const sash = useSidebarResize({
    edge,
    geometry,
    width: () => state.width,
    onResize: (value) => (state.width = value),
    onCommit: () => state.commits++,
  });
  return { sash, state };
}

function pointer(clientX: number, button = 0) {
  const element = document.createElement("div");
  const captured: number[] = [];
  element.setPointerCapture = (id: number) => captured.push(id);
  const event = {
    button,
    clientX,
    currentTarget: element,
    pointerId: 7,
    preventDefault: () => undefined,
  } as unknown as PointerEvent;
  return { captured, event };
}

describe("sidebar sash gesture", () => {
  it("tracks the pointer and takes pointer capture so the gesture ends on the handle", () => {
    const { sash, state } = createSash("leading");
    const down = pointer(500);

    sash.begin(down.event);
    expect(sash.dragging.value).toBe(true);
    expect(down.captured).toEqual([7]);

    sash.move(pointer(540).event);
    expect(state.width).toBe(280);
    sash.move(pointer(520).event);
    expect(state.width).toBe(260);
  });

  it("measures every frame from the gesture origin, so the width cannot drift", () => {
    const { sash, state } = createSash("leading", 220);
    sash.begin(pointer(400).event);

    sash.move(pointer(420).event);
    sash.move(pointer(420).event);
    sash.move(pointer(420).event);

    expect(state.width).toBe(240);
  });

  it("grows a trailing sidebar as the pointer moves the other way", () => {
    const { sash, state } = createSash("trailing", 240);
    sash.begin(pointer(900).event);

    sash.move(pointer(860).event);

    expect(state.width).toBe(280);
  });

  it("clamps at the geometry instead of collapsing, so a drag never removes the handle", () => {
    const { sash, state } = createSash("leading", 240);
    sash.begin(pointer(500).event);

    sash.move(pointer(0).event);
    expect(state.width).toBe(geometry.minSize);

    sash.move(pointer(5000).event);
    expect(state.width).toBe(geometry.maxSize);
  });

  it("commits once the pointer is released, and ignores a release it never owned", () => {
    const { sash, state } = createSash("leading");

    sash.end();
    expect(state.commits).toBe(0);

    sash.begin(pointer(500).event);
    sash.move(pointer(520).event);
    sash.end();

    expect(state.commits).toBe(1);
    expect(sash.dragging.value).toBe(false);
  });

  it("states the gesture on the document, because the pointer spends it over the work area", () => {
    const { sash } = createSash("leading");
    expect(document.documentElement.hasAttribute("data-sash-dragging")).toBe(false);

    sash.begin(pointer(500).event);
    expect(document.documentElement.hasAttribute("data-sash-dragging")).toBe(true);

    sash.end();
    expect(document.documentElement.hasAttribute("data-sash-dragging")).toBe(false);
  });

  it("ignores moves outside a gesture and non-primary buttons", () => {
    const { sash, state } = createSash("leading", 240);

    sash.move(pointer(900).event);
    expect(state.width).toBe(240);

    sash.begin(pointer(500, 2).event);
    expect(sash.dragging.value).toBe(false);
  });

  it("resizes from the keyboard in the direction the sidebar grows", () => {
    const { sash, state } = createSash("leading", 240);

    sash.keydown(new KeyboardEvent("keydown", { key: "ArrowRight" }));
    expect(state.width).toBe(256);
    sash.keydown(new KeyboardEvent("keydown", { key: "ArrowLeft" }));
    expect(state.width).toBe(240);

    sash.keydown(new KeyboardEvent("keydown", { key: "Home" }));
    expect(state.width).toBe(geometry.minSize);
    sash.keydown(new KeyboardEvent("keydown", { key: "End" }));
    expect(state.width).toBe(geometry.maxSize);

    expect(state.commits).toBe(4);
  });

  it("inverts keyboard direction for a trailing sidebar", () => {
    const { sash, state } = createSash("trailing", 240);

    sash.keydown(new KeyboardEvent("keydown", { key: "ArrowLeft" }));

    expect(state.width).toBe(256);
  });

  it("leaves unrelated keys to the application", () => {
    const { sash, state } = createSash("leading", 240);

    sash.keydown(new KeyboardEvent("keydown", { key: "Enter" }));

    expect(state.width).toBe(240);
    expect(state.commits).toBe(0);
  });
});
