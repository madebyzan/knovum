import { StarterKit } from "@tiptap/starter-kit";
import { Editor } from "@tiptap/vue-3";
import { describe, expect, it } from "vitest";

import { filterSlashCommands } from "../slash-commands";
import {
  BlockKeyboardReorder,
  deleteTopLevelBlock,
  duplicateTopLevelBlock,
  insertParagraphBelowAndSlash,
  moveTopLevelBlock,
  moveTopLevelBlockToIndex,
  selectTopLevelBlockText,
  turnTopLevelBlockInto,
} from "../tiptap-block-actions";
import { hitTestHoveredBlock, resolveBlockDropSlot, type TopLevelBlockMetric } from "../tiptap-block-gutter";
import { createCalloutExtension } from "../tiptap-callout";
import { CODE_BLOCK_LANGUAGES, matchCodeLanguage } from "../tiptap-code-block-languages";

function createEditor(html: string) {
  return new Editor({
    extensions: [StarterKit, createCalloutExtension(false), BlockKeyboardReorder],
    content: html,
  });
}

describe("top-level block actions, pointer drag geometry, and slash command search", () => {
  it("moves, reorders by index, duplicates, selects, converts, and deletes top-level blocks cleanly", () => {
    const editor = createEditor("<p>First</p><p>Second</p><p>Third</p>");
    editor.commands.setTextSelection(10);

    expect(moveTopLevelBlock(editor, -1)).toBe(true);
    expect(editor.getText()).toBe("Second\n\nFirst\n\nThird");

    expect(moveTopLevelBlockToIndex(editor, 0, 3)).toBe(true);
    expect(editor.getText()).toBe("First\n\nThird\n\nSecond");

    expect(moveTopLevelBlockToIndex(editor, 2, 0)).toBe(true);
    expect(editor.getText()).toBe("Second\n\nFirst\n\nThird");

    expect(duplicateTopLevelBlock(editor, 0)).toBe(true);
    expect(editor.getText()).toBe("Second\n\nSecond\n\nFirst\n\nThird");

    expect(turnTopLevelBlockInto(editor, "heading2", 0)).toBe(true);
    expect(editor.getJSON().content?.[0]?.type).toBe("heading");

    const range = selectTopLevelBlockText(editor, 0);
    expect(range).not.toBeNull();
    expect(editor.state.doc.textBetween(range!.from, range!.to, "\n")).toBe("Second");

    expect(deleteTopLevelBlock(editor, 0)).toBe(true);
    expect(editor.getText()).toBe("Second\n\nFirst\n\nThird");
    editor.destroy();
  });

  it("hit-tests hovered blocks and computes snap drop slots accurately", () => {
    const metrics: readonly TopLevelBlockMetric[] = [
      { index: 0, pos: 0, top: 0, bottom: 28, handleTop: 3, label: "First" },
      { index: 1, pos: 7, top: 38, bottom: 66, handleTop: 41, label: "Second" },
      { index: 2, pos: 15, top: 76, bottom: 104, handleTop: 79, label: "Third" },
    ];
    expect(hitTestHoveredBlock(metrics, 50)?.index).toBe(1);
    expect(hitTestHoveredBlock(metrics, 33)?.index).toBe(0);
    expect(resolveBlockDropSlot(metrics, 10)).toEqual({ targetIndex: 0, lineTop: -4 });
    expect(resolveBlockDropSlot(metrics, 45)).toEqual({ targetIndex: 1, lineTop: 33 });
    expect(resolveBlockDropSlot(metrics, 115)).toEqual({ targetIndex: 3, lineTop: 108 });
  });

  it("inserts a paragraph below with slash trigger for the Notion + button", () => {
    const editor = createEditor("<p>Intro</p>");
    expect(insertParagraphBelowAndSlash(editor, 0)).toBe(true);
    expect(editor.getText()).toBe("Intro\n\n/");
    editor.destroy();
  });

  it("keeps an empty paragraph when deleting the final remaining block", () => {
    const editor = createEditor("<p>Only block</p>");
    expect(deleteTopLevelBlock(editor)).toBe(true);
    expect(editor.state.doc.childCount).toBe(1);
    expect(editor.getText()).toBe("");
    editor.destroy();
  });

  it("matches slash commands across English and Chinese aliases", () => {
    const byCode = filterSlashCommands("代码");
    expect(byCode.some((item) => item.id === "code-block")).toBe(true);

    const byAi = filterSlashCommands("ask");
    expect(byAi[0]?.id).toBe("ai-ask");

    const byMath = filterSlashCommands("公式");
    expect(byMath.some((item) => item.id === "math-block")).toBe(true);
    expect(byMath.some((item) => item.id === "math-inline")).toBe(true);

    const byMermaid = filterSlashCommands("流程图");
    expect(byMermaid.some((item) => item.id === "mermaid")).toBe(true);

    expect(CODE_BLOCK_LANGUAGES.find((item) => matchCodeLanguage(item, "ts"))?.value).toBe("typescript");
    expect(CODE_BLOCK_LANGUAGES.find((item) => matchCodeLanguage(item, "py"))?.value).toBe("python");
    expect(CODE_BLOCK_LANGUAGES.find((item) => matchCodeLanguage(item, "mmd"))?.value).toBe("mermaid");
  });
});
