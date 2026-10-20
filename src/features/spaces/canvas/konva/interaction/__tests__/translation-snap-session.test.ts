import { describe, expect, it } from "vitest";

import type { CanvasNodeFrame } from "../../../schema/document";
import { createCanvasTranslationSnapIndex } from "../canvas-translation-snap-index";
import { createTranslationSnapSession } from "../translation-snap-session";

function frame(x: number, y = 0): CanvasNodeFrame {
  return { x, y, width: 100, height: 50, rotation: 0 };
}

function session() {
  return createTranslationSnapSession([frame(0)], createCanvasTranslationSnapIndex([frame(200), frame(205)]));
}

describe("translation snap gesture", () => {
  it("does not switch to a closer candidate while its original target remains held", () => {
    const snap = session();
    expect(snap({ x: 97, y: 0 }, 1).x).toBe(100);
    const positions = [101, 103, 105, 107, 105, 103, 99].map((x) => snap({ x, y: 0 }, 1).x);
    expect(positions).toEqual(positions.map(() => 100));
  });

  it("releases beyond the hysteresis radius and acquires a new target", () => {
    const snap = session();
    snap({ x: 97, y: 0 }, 1);
    expect(snap({ x: 110, y: 0 }, 1).x).toBe(100);
    expect(snap({ x: 111, y: 0 }, 1).x).toBe(105);
    expect(snap({ x: 116, y: 0 }, 1).x).toBe(116);
  });

  it("keeps reference lines unchanged during small pointer noise", () => {
    const snap = session();
    const guides = snap({ x: 97, y: 0 }, 1).guides;
    for (const x of [99, 101, 103, 100]) {
      expect(snap({ x, y: 0 }, 1).guides).toEqual(guides);
    }
  });

  it("acquires and releases symmetrically in screen pixels across zoom levels", () => {
    for (const zoom of [0.5, 1, 2]) {
      const snap = createTranslationSnapSession([frame(0)], createCanvasTranslationSnapIndex([frame(200)]));
      expect(snap({ x: 100 - 5 / zoom, y: 0 }, zoom).x).toBe(100);
      expect(snap({ x: 100 + 9 / zoom, y: 0 }, zoom).x).toBe(100);
      expect(snap({ x: 100 + 11 / zoom, y: 0 }, zoom).x).toBe(100 + 11 / zoom);
    }
  });

  it("a new gesture does not inherit the previous gesture's locks", () => {
    const first = session();
    first({ x: 97, y: 0 }, 1);
    expect(first({ x: 105, y: 0 }, 1).x).toBe(100);
    expect(session()({ x: 105, y: 0 }, 1).x).toBe(105);
  });

  it("releases spacing when the pointer leaves the participating row", () => {
    const snap = createTranslationSnapSession(
      [frame(300)],
      createCanvasTranslationSnapIndex([frame(0), frame(150)]),
    );
    expect(snap({ x: 3, y: 0 }, 1).guides.filter(({ kind }) => kind === "spacing")).toHaveLength(2);
    expect(snap({ x: 3, y: 100 }, 1).guides.filter(({ kind }) => kind === "spacing")).toHaveLength(0);
  });
});
