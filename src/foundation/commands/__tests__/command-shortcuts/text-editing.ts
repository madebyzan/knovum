import { beforeEach, describe, expect, it } from "vitest";

import { bindCommandShortcuts } from "../../command-shortcuts";
import { command, mounted, press, settle, shortcutHost } from "../support/shortcut-harness";

/** Chords the text being edited owns instead of the application. */
export function textEditingScenarios() {
  describe("platform text editing chords", () => {
    beforeEach(() => {
      shortcutHost.platform.mockReturnValue("windows");
      document.body.replaceChildren();
    });

    it("leaves text editing to the text being edited, even for a command allowed while typing", async () => {
      const documentUndo = command("$mod+Z", { allowInEditable: true });
      const unbind = bindCommandShortcuts([documentUndo]);
      const macLike = /Mac|iPod|iPhone|iPad/.test(navigator.platform);
      const chord = { key: "z", code: "KeyZ", metaKey: macLike, ctrlKey: !macLike };

      const field = mounted("input");
      const richText = mounted("div", { contenteditable: "true" });
      const inField = press(field, chord);
      const inRichText = press(richText, chord);
      await settle();

      expect(documentUndo.run).not.toHaveBeenCalled();
      expect(inField.defaultPrevented).toBe(false);
      expect(inRichText.defaultPrevented).toBe(false);

      press(mounted("div"), chord);
      await settle();
      expect(documentUndo.run).toHaveBeenCalledOnce();
      unbind();
    });
  });
}
