import { Editor } from "@tiptap/core";
import { StarterKit } from "@tiptap/starter-kit";
import { expect, it, onTestFinished } from "vitest";

import { documentEditCommands, registerDocumentEditCommands } from "../document-edit-commands";

it("undoes the document through its command port instead of canvas history", () => {
  const editor = new Editor({ extensions: [StarterKit], content: "<p>Original</p>" });
  onTestFinished(() => editor.destroy());
  onTestFinished(
    registerDocumentEditCommands({
      canUndo: () => editor.can().undo(),
      canRedo: () => editor.can().redo(),
      undo: () => {
        editor.commands.undo();
      },
      redo: () => {
        editor.commands.redo();
      },
      find: () => {},
      hasOutline: () => true,
      focusOutline: () => {},
      save: async () => {},
      close: async () => {},
    }),
  );
  editor.commands.insertContent("Changed ");
  const undo = documentEditCommands().find(({ id }) => id === "document.undo")!;
  expect(undo.availability?.()).toEqual({ state: "ready" });
  undo.run();
  expect(editor.getText()).toBe("Original");
  expect(documentEditCommands().map(({ id }) => id)).not.toContain("spaces.zoom-in");
});

it("does not offer document commands while its editor is loading", () => {
  expect(documentEditCommands()).toEqual([]);
});
