import { describe, expect, it } from "vitest";
import { effectScope, shallowRef } from "vue";

import { splitClip } from "../clip-edits";
import { useVideoEditSelection } from "../presentation/use-video-edit-selection";
import { addTextOverlay, createVideoEditProject } from "../project";
import { useVideoEditHistory } from "../use-video-edit-history";

describe("video editor selection and history", () => {
  it.each(["clip", "text"] as const)("commits %s deletion as one undoable edit", (kind) => {
    const scope = effectScope();
    try {
      scope.run(() => {
        const source = createVideoEditProject({
          nodeId: "source",
          sourceRevision: "source",
          durationMs: 2_000,
          sourceWidth: 1280,
          sourceHeight: 720,
        });
        const initial = addTextOverlay(splitClip(source, 1_000), 200, "Arial", "Text");
        const project = shallowRef(initial);
        const history = useVideoEditHistory(project);
        history.reset(initial);
        const selection = useVideoEditSelection(project);
        if (kind === "clip") selection.selectClip(initial.videoTrack.clips[0]!.id);
        else selection.selectText(initial.textTrack.items[0]!.id);

        const next = selection.removeSelection();
        expect(project.value).toBe(initial);
        expect(next).toBeDefined();
        history.commit(next!);
        expect(history.canUndo.value).toBe(true);
        history.undo();
        expect(project.value).toEqual(initial);
      });
    } finally {
      scope.stop();
    }
  });
});
