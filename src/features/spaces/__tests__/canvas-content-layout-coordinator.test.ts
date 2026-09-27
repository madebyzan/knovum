import { describe, expect, it, vi } from "vitest";

import type { CanvasEditorPort } from "../canvas/contracts/canvas-editor-port";
import { createCanvasContentLayoutCoordinator } from "../presentation/canvas/canvas-content-layout-coordinator";

describe("canvas content layout coordinator", () => {
  it("measures and flushes content height before its session phase resolves", async () => {
    const calls: string[] = [];
    const editor = {
      fitNodeContentHeight() {
        calls.push("fit");
      },
      flushContentHeightFitting() {
        calls.push("flush");
        return true;
      },
      setContentHeightFittingPaused: vi.fn<(paused: boolean) => void>(),
    } as unknown as CanvasEditorPort;
    const coordinator = createCanvasContentLayoutCoordinator({
      directManipulation: () => false,
      editor: () => editor,
      projectDirectManipulation() {},
      projectMutationPending() {},
    });
    coordinator.measurements.register(() => {
      calls.push("measure");
      coordinator.fit("text", {
        contentRevision: null,
        height: 64,
        sizeAuthority: "frame",
        typographyScale: 1,
        width: 240,
      });
    });

    await coordinator.settle();

    expect(calls).toEqual(["measure", "fit", "flush"]);
  });
});
