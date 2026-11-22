// @vitest-environment happy-dom

import { Editor } from "@tiptap/core";
import { describe, expect, it, vi } from "vitest";

import { unescapeCitationMarkers } from "../../knowledge/model/citation-markers";
import { createSpaceTiptapExtensions } from "../tiptap-extensions";

function createEditor(content: string, openCitation = vi.fn<(index: number) => void>()) {
  const editor = new Editor({
    extensions: createSpaceTiptapExtensions("", vi.fn<(error: unknown) => void>(), openCitation),
    content,
    contentType: "markdown",
  });
  return { editor, openCitation };
}

describe("createGroundedCitationExtension", () => {
  it("renders grounded citations using the shared ui-citation button widget and skips code", () => {
    const { editor, openCitation } = createEditor("Claim [C5][C43] with `[C2]`.\n\n```txt\n[C9]\n```\n");
    try {
      const badges = Array.from(editor.view.dom.querySelectorAll<HTMLButtonElement>("button.ui-citation"));
      expect(badges.map((badge) => badge.textContent)).toEqual(["5", "43"]);
      expect(badges.map((badge) => badge.dataset.citationIndex)).toEqual(["5", "43"]);
      expect(editor.view.dom.querySelectorAll(".grounded-citation-source")).toHaveLength(2);
      expect(unescapeCitationMarkers(editor.getMarkdown())).toContain("[C5][C43]");

      badges[0]!.click();
      expect(openCitation).toHaveBeenCalledWith(5);
    } finally {
      editor.destroy();
    }
  });
});
