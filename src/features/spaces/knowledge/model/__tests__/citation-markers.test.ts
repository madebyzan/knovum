import { describe, expect, it } from "vitest";

import { stripCitationMarkers, unescapeCitationMarkers } from "../citation-markers";

describe("citation-markers", () => {
  it("strips internal citation markers while keeping links, invalid tokens, and code intact", () => {
    const input = [
      "Revenue grew 12% [C1][C2]. Keep [links](https://a.b), [C0], [Cx], and `[C3]`.",
      "```md",
      "Example [C4]",
      "```",
    ].join("\n");
    expect(stripCitationMarkers(input)).toBe(
      [
        "Revenue grew 12%. Keep [links](https://a.b), [C0], [Cx], and `[C3]`.",
        "```md",
        "Example [C4]",
        "```",
      ].join("\n"),
    );
  });

  it("restores escaped citation markers outside code spans and fences", () => {
    const input = "Claim \\[C5\\]\\[C12\\]. Code `\\[C5\\]`.\n```md\n\\[C2\\]\n```";
    expect(unescapeCitationMarkers(input)).toBe("Claim [C5][C12]. Code `\\[C5\\]`.\n```md\n\\[C2\\]\n```");
  });
});
