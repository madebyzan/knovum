import { describe, expect, it } from "vitest";

import fixtures from "../../../../tests/fixtures/image-tone.json";
import { applyImageTone } from "../image-editing/image-tone";

describe("preview and Rust export tone contract", () => {
  it.each(fixtures)("matches the shared $name pixel fixture", (fixture) => {
    const pixels = applyImageTone(new Uint8ClampedArray(fixture.source), fixture.adjustments, {
      width: fixture.width,
      height: fixture.height,
      imageRect: { left: 0, top: 0, width: fixture.width, height: fixture.height },
    });
    expect([...pixels]).toEqual(fixture.expected);
  });
});
