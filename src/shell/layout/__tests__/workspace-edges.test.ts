import { readFileSync } from "node:fs";

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, nextTick, ref } from "vue";

import { PANEL_MOTION, useWorkspaceEdges, type WorkspaceEdges } from "../use-workspace-edges";

function setup(initial: { pinned: boolean; preview?: boolean; right?: boolean }) {
  const pinned = ref(initial.pinned);
  const preview = ref(initial.preview ?? false);
  const rightOpen = ref(initial.right ?? false);
  const width = ref(280);
  const visible = ref(pinned.value || preview.value);
  const scope = effectScope();
  const edges = scope.run(() =>
    useWorkspaceEdges({
      sidebarPinned: pinned,
      sidebarVisible: visible,
      sidebarWidth: width,
      rightSidebarOpen: rightOpen,
      rightSidebarWidth: ref(320),
    }),
  )!;
  function set(next: { pinned?: boolean; preview?: boolean; right?: boolean }) {
    pinned.value = next.pinned ?? pinned.value;
    preview.value = next.preview ?? preview.value;
    rightOpen.value = next.right ?? rightOpen.value;
    visible.value = pinned.value || preview.value;
  }
  return { edges, set, width, stop: () => scope.stop() };
}

/** Frames the clock produces after a state change; the transition itself starts on the next tick. */
async function frames(edges: { value: WorkspaceEdges }, count: number) {
  await nextTick();
  return Array.from({ length: count }, () => {
    vi.advanceTimersByTime(16);
    return { ...edges.value };
  });
}

function stubReducedMotion(reduce: boolean) {
  vi.stubGlobal("matchMedia", (query: string) => ({
    matches: reduce && query.includes("reduce"),
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }));
}

describe("workspace edges", () => {
  beforeEach(() => {
    vi.useFakeTimers({ toFake: ["Date", "requestAnimationFrame", "setTimeout"] });
    stubReducedMotion(false);
  });
  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("starts where the state puts it, without motion", () => {
    const { edges, stop } = setup({ pinned: true, right: true });
    expect(edges.value).toEqual({ leading: 280, sidebar: 280, trailing: 320 });
    stop();
  });

  it("moves every edge on whole pixels, the sidebar's edge on the work area's in every frame", async () => {
    const { edges, set, stop } = setup({ pinned: true });
    set({ pinned: false });
    const seen = await frames(edges, 20);

    for (const frame of seen) {
      expect(Number.isInteger(frame.leading) && Number.isInteger(frame.sidebar)).toBe(true);
      expect(frame.sidebar).toBe(frame.leading);
    }
    expect(seen.some(({ leading }) => leading > 0 && leading < 280)).toBe(true);
    expect(seen.at(-1)).toEqual({ leading: 0, sidebar: 0, trailing: 0 });
    stop();
  });

  it("holds a previewed sidebar still while pinning makes room for it", async () => {
    const { edges, set, stop } = setup({ pinned: false, preview: true });
    set({ pinned: true, preview: false });
    const seen = await frames(edges, 20);

    expect(seen.every(({ sidebar }) => sidebar === 280)).toBe(true);
    const leading = seen.map((frame) => frame.leading);
    expect(leading).toEqual(leading.toSorted((a, b) => a - b));
    expect(seen.at(-1)!.leading).toBe(280);
    stop();
  });

  it("applies a sash width in the same frame, with no motion to suspend", async () => {
    const { edges, width, stop } = setup({ pinned: true });
    width.value = 333;
    await nextTick();
    expect(edges.value.leading).toBe(333);
    expect(edges.value.sidebar).toBe(333);
    stop();
  });

  it("jumps to the end state when reduced motion is requested", async () => {
    stubReducedMotion(true);
    const { edges, set, stop } = setup({ pinned: true });
    set({ pinned: false, right: true });
    await nextTick();
    expect(edges.value).toEqual({ leading: 0, sidebar: 0, trailing: 320 });
    stop();
  });

  it("runs on the duration and curve the CSS fades use", () => {
    const tokens = readFileSync("src/styles/tokens.css", "utf8");
    expect(tokens).toContain(`--knovum-panel-motion-duration: ${PANEL_MOTION.durationMs}ms;`);
    expect(tokens).toContain(
      `--knovum-panel-motion-easing: cubic-bezier(${PANEL_MOTION.easing.join(", ")});`,
    );
  });
});
