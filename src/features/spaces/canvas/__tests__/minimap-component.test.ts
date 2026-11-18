import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, shallowRef } from "vue";

import { i18n } from "../../../../foundation/i18n/instance";
import { openStoreSpaceTab } from "../../__tests__/support/session-context";
import { createSpaceNotePayload } from "../../document-editing/tiptap-content";
import { useSpaceStore } from "../../session/space-store";
import type { CanvasEditorPort, CanvasViewport } from "../contracts/canvas-editor-port";
import { registerCanvasEditor, updateCanvasViewportProjection } from "../editor-session";
import { createSpaceNode } from "../node";
import SpaceCanvasMinimap from "../presentation/minimap/SpaceCanvasMinimap.vue";

vi.mock("../use-canvas-palette", () => ({
  useCanvasPalette: () =>
    shallowRef({
      typography: { fontFamily: "system-ui", labelSize: 13, captionSize: 12 },
      accent: "#3b82f6",
      border: "#e2e8f0",
      borderStrong: "#cbd5e1",
      canvas: "#fafafa",
      foreground: "#111827",
      foregroundSecondary: "#6b7280",
      grid: "#e5e7eb",
      measurement: "#f59e0b",
      contentFillByPreset: {
        yellow: "#fef08a",
        green: "#bbf7d0",
        blue: "#bfdbfe",
        pink: "#fbcfe8",
        gray: "#e5e7eb",
      },
      panel: "#ffffff",
      relationByColor: {
        neutral: "#6b7280",
        blue: "#3b82f6",
        green: "#22c55e",
        amber: "#f59e0b",
        red: "#ef4444",
        purple: "#a855f7",
      },
    }),
}));

const noopMock = () => vi.fn<() => void>();

function mockCanvasContext() {
  const methods =
    "beginPath clearRect fill fillRect lineTo moveTo restore rotate roundRect save setTransform stroke translate".split(
      " ",
    );
  return Object.assign(Object.fromEntries(methods.map((method) => [method, noopMock()])), {
    globalAlpha: 1,
    fillStyle: "",
    strokeStyle: "",
    lineWidth: 1,
  }) as unknown as CanvasRenderingContext2D;
}

function noteLayout(structureNodeId: string, x: number, y: number) {
  return {
    structureNodeId,
    kind: "content" as const,
    transform: { x, y, rotation: 0, scale: 1 },
    size: { width: 200, height: 140 },
    sizingMode: "content" as const,
  };
}

describe("SpaceCanvasMinimap", () => {
  let detachEditor: (() => void) | undefined;
  let wrapper: ReturnType<typeof mount> | undefined;

  beforeEach(() => {
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(mockCanvasContext());
  });

  afterEach(() => {
    wrapper?.unmount();
    wrapper = undefined;
    detachEditor?.();
    detachEditor = undefined;
    vi.restoreAllMocks();
  });

  function setup(options: { width?: number; height?: number; open?: boolean } = {}) {
    const pinia = createPinia();
    setActivePinia(pinia);
    const store = useSpaceStore();
    openStoreSpaceTab(store, "inbox");
    if (options.open !== undefined) store.minimapOpen = options.open;
    store.nodes = [
      createSpaceNode(createSpaceNotePayload("A"), { x: 100, y: 100 }, "a"),
      createSpaceNode(createSpaceNotePayload("B"), { x: 2400, y: 120 }, "b"),
    ];
    store.editSnapshot = {
      editRevision: 1,
      structure: [
        { id: "c-a", kind: "content", nodeId: "a", parentId: null, order: 0, locked: false },
        { id: "c-b", kind: "content", nodeId: "b", parentId: null, order: 1, locked: false },
      ],
      layouts: [noteLayout("c-a", 100, 100), noteLayout("c-b", 2400, 120)],
      relationStyles: [],
    };
    const navigateViewport = vi.fn<(next: CanvasViewport) => void>((next) =>
      updateCanvasViewportProjection(next),
    );
    const setViewport = vi.fn<(next: CanvasViewport) => void>((next) => updateCanvasViewportProjection(next));
    const zoomToFit = vi.fn<() => void>();
    detachEditor = registerCanvasEditor({
      getViewport: () => ({ x: 0, y: 0, zoom: 1 }),
      getSelectionSnapshot: () => ({
        nodeIds: [],
        structureNodeIds: [],
        relationIds: [],
        nodePlacements: [],
      }),
      navigateViewport,
      setViewport,
      zoomToFit,
    } as unknown as CanvasEditorPort);
    wrapper = mount(SpaceCanvasMinimap, {
      props: {
        navigationEnabled: true,
        hostSize: { width: options.width ?? 1200, height: options.height ?? 800 },
        relations: [],
      },
      global: { plugins: [pinia, i18n] },
    });
    return { store, navigateViewport, setViewport, zoomToFit, wrapper };
  }

  it("starts collapsed by default and expands smoothly on toggle click", async () => {
    const { store, wrapper: mounted } = setup();
    expect(store.minimapOpen).toBe(false);
    expect(mounted.find('[role="region"]').exists()).toBe(false);
    await mounted.find("button").trigger("click");
    await nextTick();
    expect(store.minimapOpen).toBe(true);
    expect(mounted.find('[role="region"]').exists()).toBe(true);
  });

  it("supports click navigation, scrub, wheel pan, double-click fit, and collapse when open", async () => {
    const { store, navigateViewport, setViewport, zoomToFit, wrapper: mounted } = setup({ open: true });
    const surface = mounted.find('[role="region"]');
    expect(surface.exists()).toBe(true);
    vi.spyOn(surface.element, "getBoundingClientRect").mockReturnValue({
      x: 0,
      y: 0,
      left: 0,
      top: 0,
      width: 184,
      height: 112,
      right: 184,
      bottom: 112,
      toJSON: () => ({}),
    });
    await surface.trigger("pointerdown", { button: 0, pointerId: 1, clientX: 160, clientY: 60 });
    expect(navigateViewport).toHaveBeenCalledTimes(1);
    await surface.trigger("pointermove", { button: 0, pointerId: 1, clientX: 170, clientY: 70 });
    expect(setViewport).toHaveBeenCalled();
    await surface.trigger("pointerup", { button: 0, pointerId: 1, clientX: 170, clientY: 70 });
    await surface.trigger("wheel", { deltaX: 20, deltaY: 10, deltaMode: 0 });
    expect(setViewport).toHaveBeenCalled();
    await surface.trigger("dblclick");
    expect(zoomToFit).toHaveBeenCalledTimes(1);
    await mounted.findAll("button").at(-1)!.trigger("click");
    await nextTick();
    expect(store.minimapOpen).toBe(false);
    expect(mounted.find('[role="region"]').exists()).toBe(false);
  });

  it("suppresses panel when recording dock would collide on narrow host", async () => {
    const { store, wrapper: mounted } = setup({ width: 640, height: 700, open: true });
    expect(mounted.find('[role="region"]').exists()).toBe(true);
    store.audioRecordingTarget = { spaceId: "inbox", position: { x: 0, y: 0 } };
    await nextTick();
    expect(mounted.find('[role="region"]').exists()).toBe(false);
    expect(mounted.find("button").exists()).toBe(true);
  });
});
