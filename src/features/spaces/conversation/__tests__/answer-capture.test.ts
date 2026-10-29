import { describe, expect, it } from "vitest";

import { answerTitle, markdownFromAnswer, quotedFollowUp, withoutCitationMarkers } from "../answer-capture";

describe("answer capture", () => {
  it("drops citation markers, which mean nothing outside their answer", () => {
    expect(withoutCitationMarkers("Revenue grew 12% [C1][C2]. Costs fell [C3].")).toBe(
      "Revenue grew 12%. Costs fell.",
    );
    expect(withoutCitationMarkers("Keep [links](https://a.b) and [C0] and [Cx]")).toBe(
      "Keep [links](https://a.b) and [C0] and [Cx]",
    );
  });

  it("titles a node after the first line of the answer, without Markdown", () => {
    expect(answerTitle("\n## **Q3** results [C1]\n\nBody", "Untitled")).toBe("Q3 results");
    expect(answerTitle("- [Plan A](https://x.y) is cheaper", "Untitled")).toBe("Plan A is cheaper");
    expect(answerTitle("Here are the options:\n1. One", "Untitled")).toBe("Here are the options");
    expect(answerTitle("   \n```\n", "Untitled")).toBe("Untitled");
    expect(answerTitle("x".repeat(200), "Untitled")).toHaveLength(80);
  });

  it("keeps the answer's Markdown as the node text", () => {
    expect(markdownFromAnswer("# Plan\n\n| A | B |\n|---|---|\n| 1 | 2 [C1] |\n", "Untitled")).toEqual({
      kind: "markdown",
      title: "Plan",
      text: "# Plan\n\n| A | B |\n|---|---|\n| 1 | 2 |",
    });
  });

  it("quotes the selection above the draft", () => {
    expect(quotedFollowUp("  First line\n\nSecond line  ", "")).toBe("> First line\n>\n> Second line\n\n");
    expect(quotedFollowUp("Point", "  Expand on this ")).toBe("> Point\n\nExpand on this");
  });
});
