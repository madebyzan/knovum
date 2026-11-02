import { describe, expect, it } from "vitest";

import { pdfFindAnchor } from "../pdf-find-anchor";

describe("pdfFindAnchor", () => {
  it("keeps a short passage whole and flattens its whitespace", () => {
    expect(pdfFindAnchor("  payment is\n due  in 30 days ")).toBe("payment is due in 30 days");
  });

  it("cuts a long passage on a word boundary", () => {
    const anchor = pdfFindAnchor("word ".repeat(40));
    expect(anchor.length).toBeLessThanOrEqual(48);
    expect(anchor.endsWith("word")).toBe(true);
  });

  it("cuts text without spaces at the character budget", () => {
    expect(pdfFindAnchor("付".repeat(100))).toHaveLength(48);
  });
});
