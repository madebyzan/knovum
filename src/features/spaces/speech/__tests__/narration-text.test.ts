import { expect, it } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { narrationText } from "../narration-text";

function markdown(text: string): SpaceCanvasNode {
  return {
    id: "source",
    type: "markdown",
    initialPosition: { x: 0, y: 0 },
    data: { kind: "markdown", title: "Source file", text },
  };
}

it("keeps headings, prose and line breaks without reading Markdown formatting or URLs", () => {
  const text = narrationText(
    markdown(
      "# Review\n\n**Important** [decision](https://example.com)\nNext line.\n\n- First item\n- Second item",
    ),
  );
  expect(text).toContain("Review");
  expect(text).toContain("Important decision\nNext line.");
  expect(text).toContain("First item");
  expect(text).not.toContain("https://");
  expect(text).not.toContain("Source file");
  expect(text).not.toContain("**");
});

it("reads a heading-only manuscript without losing its content", () => {
  expect(narrationText(markdown("# Hello"))).toBe("Hello");
  expect(narrationText(markdown(""))).toBe("");
});
