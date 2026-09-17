import { describe, expect, it } from "vitest";

import { sliceUtf16Safe, truncateUtf16 } from "../model/text";

describe("truncateUtf16", () => {
  it("never splits a surrogate pair", () => {
    expect(truncateUtf16("abc😀z", 4)).toBe("abc");
    expect(truncateUtf16("abc😀z", 5)).toBe("abc😀");
  });

  it("uses JavaScript UTF-16 code-unit limits", () => {
    expect(truncateUtf16("你好世界", 3)).toBe("你好世");
    expect(truncateUtf16("value", 0)).toBe("");
  });
});

describe("sliceUtf16Safe", () => {
  it("moves interior boundaries away from surrogate halves", () => {
    expect(sliceUtf16Safe("a😀bc", 2, 5)).toBe("bc");
    expect(sliceUtf16Safe("a😀bc", 0, 2)).toBe("a");
    expect(sliceUtf16Safe("a😀bc", 1, 3)).toBe("😀");
  });
});
