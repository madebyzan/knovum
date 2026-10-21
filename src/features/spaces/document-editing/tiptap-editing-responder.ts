import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";

import { registerEditorHistory } from "../../../foundation/commands/editing-responder";

/**
 * Declares the editor's own undo history as the responder for undo and redo while its text has
 * focus, so the standard editing chords never reach past the text into the canvas or the platform.
 * The plugin view lives exactly as long as the editable element it registers.
 */
export const EditingResponder = Extension.create({
  name: "editingResponder",
  addProseMirrorPlugins() {
    const editor = this.editor;
    return [
      new Plugin({
        key: new PluginKey("editingResponder"),
        view: (view) => ({
          destroy: registerEditorHistory(view.dom, {
            undo: () => editor.commands.undo(),
            redo: () => editor.commands.redo(),
          }),
        }),
      }),
    ];
  },
});
