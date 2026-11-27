import { describe, expect, it } from "vitest";

import {
  cleanSerializedMarkdown,
  extractImportedMarkdownDocument,
  formatExportMarkdown,
} from "../markdown-document";

describe("markdown-document codec", () => {
  describe("extractImportedMarkdownDocument", () => {
    it("extracts leading H1 heading as title and strips it from body", () => {
      expect(
        extractImportedMarkdownDocument(
          "document.md",
          "# My Document Title\n\nFirst paragraph of text.\n\nSecond paragraph.",
        ),
      ).toEqual({
        title: "My Document Title",
        text: "First paragraph of text.\n\nSecond paragraph.",
      });
    });

    it("handles closing hashes and leading blank lines before H1", () => {
      expect(
        extractImportedMarkdownDocument("guide.markdown", "\n\n# User Guide ###\n\nContent starts here."),
      ).toEqual({
        title: "User Guide",
        text: "Content starts here.",
      });
    });

    it("preserves body and falls back to filename when no leading H1 is present", () => {
      expect(extractImportedMarkdownDocument("notes.md", "## Subheading\n\nBody content.")).toEqual({
        title: "notes",
        text: "## Subheading\n\nBody content.",
      });
    });

    it("handles title-only documents cleanly", () => {
      expect(extractImportedMarkdownDocument("standalone.md", "# Standalone Heading\n")).toEqual({
        title: "Standalone Heading",
        text: "",
      });
    });
  });

  describe("formatExportMarkdown", () => {
    it("prepends title heading when markdown does not start with an H1", () => {
      expect(formatExportMarkdown("Report", "Body content here.")).toBe("# Report\n\nBody content here.");
    });

    it("does not duplicate title when markdown already starts with an H1", () => {
      expect(formatExportMarkdown("Report", "# Existing Title\n\nBody content here.")).toBe(
        "# Existing Title\n\nBody content here.",
      );
    });

    it("exports only the heading when markdown body is empty", () => {
      expect(formatExportMarkdown("Empty Doc", "")).toBe("# Empty Doc\n");
    });

    it("returns markdown as-is when title is blank", () => {
      expect(formatExportMarkdown("   ", "Body content.")).toBe("Body content.");
    });

    it("strips internal citation anchors cleanly when exporting a grounded document", () => {
      const exported = formatExportMarkdown("Report", "Key insight [C3][C5]. Keep `[C3]` literal.", {
        stripCitations: true,
      });
      expect(exported).toBe("# Report\n\nKey insight. Keep `[C3]` literal.");
    });

    it("preserves non-grounded markdown verbatim when stripCitations is false", () => {
      expect(formatExportMarkdown("Report", "Body [C1].", { stripCitations: false })).toBe(
        "# Report\n\nBody [C1].",
      );
    });

    it("strips Tiptap &nbsp; empty-paragraph artifacts outside code fences", () => {
      const raw = "&nbsp;\n\n&nbsp;\n\nHello\n\n&nbsp;\n\n```html\n&nbsp;\n```\n\nWorld";
      expect(cleanSerializedMarkdown(raw)).toBe("Hello\n\n```html\n&nbsp;\n```\n\nWorld");
      expect(formatExportMarkdown("Report", "&nbsp;\n\n&nbsp;\n\nBody")).toBe("# Report\n\nBody");
    });
  });
});
