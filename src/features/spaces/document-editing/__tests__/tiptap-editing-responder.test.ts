import { Editor } from "@tiptap/core";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  resolveEditingResponder,
  runEditingCommand,
} from "../../../../foundation/commands/editing-responder";
import { registerAppCommands } from "../../../../foundation/commands/run-app-command";
import type { AppCommand } from "../../../../shared/contracts/app-command";
import { createCanvasTextExtensions } from "../tiptap-extensions";

vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

afterEach(() => {
  registerAppCommands([]);
  document.body.replaceChildren();
});

describe("Tiptap editing responder", () => {
  it("undoes canvas text in the text itself, not on the canvas", async () => {
    const canvasUndo: AppCommand = {
      id: "spaces.undo",
      title: "Undo",
      editingIntent: "undo",
      run: vi.fn<() => void>(),
    };
    registerAppCommands([canvasUndo]);
    const element = document.createElement("div");
    document.body.append(element);
    const editor = new Editor({
      element,
      content: "<p>Note</p>",
      extensions: createCanvasTextExtensions(""),
    });
    editor.commands.insertContentAt(editor.state.doc.content.size - 1, " edited");
    editor.view.dom.focus();
    expect(editor.getText()).toBe("Note edited");

    await runEditingCommand("undo");
    await settle();

    expect(editor.getText()).toBe("Note");
    expect(canvasUndo.run).not.toHaveBeenCalled();
    editor.destroy();
  });

  it("stops answering once the editor is gone", () => {
    const element = document.createElement("div");
    document.body.append(element);
    const editor = new Editor({
      element,
      content: "<p>Note</p>",
      extensions: createCanvasTextExtensions(""),
    });
    const editable = editor.view.dom;
    expect(resolveEditingResponder("undo", [], editable).kind).toBe("editor-history");

    editor.destroy();

    expect(resolveEditingResponder("undo", [], editable).kind).not.toBe("editor-history");
  });
});
