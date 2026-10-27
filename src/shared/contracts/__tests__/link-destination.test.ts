import { describe, expect, it } from "vitest";

import { linkDestination } from "../link-destination";

describe("linkDestination", () => {
  it("hands web pages and mail addresses to the system", () => {
    for (const href of ["https://example.com/a", "HTTP://example.com", "mailto:a@example.com"])
      expect(linkDestination(href)).toBe("external");
  });

  it("reads a path, absolute, relative or on a Windows drive, as a local file", () => {
    for (const href of [
      "/Users/zand/agent-workspaces/PPT%E6%91%98%E8%A6%81.md",
      "notes/summary.md",
      "./summary.md#L3",
      "C:%5CUsers%5Czand%5Csummary.md",
      "C:/Users/zand/summary.md",
    ])
      expect(linkDestination(href)).toBe("local");
  });

  it("leads nowhere for an anchor, another host without a scheme, or another scheme", () => {
    for (const href of [
      "",
      "#section",
      "//example.com/x",
      "tauri://localhost/",
      "javascript:alert(1)",
      "ftp://x",
    ])
      expect(linkDestination(href)).toBeUndefined();
  });
});
