import { describe, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import { insertMaterial } from "../edits/material";
import { addTextOverlay } from "../edits/text";
import { applyTransition } from "../edits/transition";
import { setClipVolume } from "../edits/trim";
import { useVideoSelection } from "../presentation/use-video-selection";
import { createVideoEditProject, defaultTextStyle, type VideoAsset } from "../project";
import { retainSelection, selectionSpan, type VideoSelection } from "../selection";
import { useVideoEditHistory } from "../use-video-edit-history";

const second: VideoAsset = {
  nodeId: "second",
  sourceRevision: "second-revision",
  kind: "audio",
  durationMs: 10_000,
  width: 0,
  height: 0,
};

function fixture() {
  const source = createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 4_000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
  return addTextOverlay(insertMaterial(source, second, 1_000), 2_000, defaultTextStyle("Arial"), "Text");
}

function runSelection(project: ReturnType<typeof fixture>) {
  const scope = effectScope();
  const current = shallowRef(project);
  const selection = shallowRef<VideoSelection>();
  const seek = vi.fn<(timeMs: number) => void>();
  const pause = vi.fn<() => void>();
  const playheadMs = shallowRef(0);
  const controller = scope.run(() =>
    useVideoSelection({
      project: current,
      selection,
      seek,
      pause,
      playheadMs: () => playheadMs.value,
    }),
  )!;
  return { scope, controller, current, seek, pause, playheadMs };
}

describe("selection rules", () => {
  it("reports the timeline span of every kind of selection", () => {
    const project = applyTransition(fixture(), fixture().videoTrack.clips[0]!.id, "dissolve", 500);
    const clip = project.videoTrack.clips[0]!;
    expect(selectionSpan(project, { kind: "clip", id: clip.id })).toEqual({
      startMs: 0,
      endMs: 4_000,
    });
    const audio = project.audioTrack.clips[0]!;
    expect(selectionSpan(project, { kind: "audio", id: audio.id })).toMatchObject({
      startMs: 1_000,
    });
    const text = project.textTrack.items[0]!;
    expect(selectionSpan(project, { kind: "text", id: text.id })).toEqual({
      startMs: text.startMs,
      endMs: text.endMs,
    });
    expect(selectionSpan(project, { kind: "transition", id: "missing" })).toBeUndefined();
  });

  it("drops a selection whose subject disappeared from the project", () => {
    const project = fixture();
    const clip = project.videoTrack.clips[0]!;
    expect(retainSelection(project, { kind: "clip", id: clip.id })).toMatchObject({ id: clip.id });
    expect(retainSelection(project, { kind: "clip", id: "gone" })).toBeUndefined();
  });

  it("reveals a selection that sits away from the playhead", () => {
    const project = fixture();
    const { scope, controller, seek, pause, playheadMs } = runSelection(project);
    try {
      playheadMs.value = 500;
      controller.select({ kind: "text", id: project.textTrack.items[0]!.id });
      expect(pause).toHaveBeenCalledOnce();
      expect(seek).toHaveBeenCalledWith(project.textTrack.items[0]!.startMs);
      seek.mockClear();
      playheadMs.value = 2_500;
      controller.select({ kind: "text", id: project.textTrack.items[0]!.id });
      expect(seek).not.toHaveBeenCalled();
    } finally {
      scope.stop();
    }
  });

  it("keeps deletion of the last visual clip unavailable", () => {
    const project = fixture();
    const { scope, controller } = runSelection(project);
    try {
      controller.select({ kind: "clip", id: project.videoTrack.clips[0]!.id }, { reveal: false });
      expect(controller.canDelete.value).toBe(false);
      expect(controller.removeSelected()).toBeUndefined();
      controller.select({ kind: "text", id: project.textTrack.items[0]!.id }, { reveal: false });
      expect(controller.canDelete.value).toBe(true);
      expect(controller.removeSelected()?.textTrack.items).toEqual([]);
    } finally {
      scope.stop();
    }
  });
});

describe("edit history", () => {
  it("commits a deletion as one undoable edit without mutating the current project", () => {
    const scope = effectScope();
    try {
      scope.run(() => {
        const initial = fixture();
        const project = shallowRef(initial);
        const history = useVideoEditHistory(project);
        history.reset(initial);
        const controller = useVideoSelection({
          project,
          selection: shallowRef<VideoSelection>(),
          seek: vi.fn<(timeMs: number) => void>(),
          pause: vi.fn<() => void>(),
          playheadMs: () => 0,
        });
        controller.select({ kind: "text", id: initial.textTrack.items[0]!.id }, { reveal: false });
        const next = controller.removeSelected()!;
        expect(project.value).toBe(initial);
        history.commit(next);
        expect(history.canUndo.value).toBe(true);
        expect(history.dirty.value).toBe(true);
        history.undo();
        expect(project.value).toEqual(initial);
        expect(history.dirty.value).toBe(false);
      });
    } finally {
      scope.stop();
    }
  });

  it("merges consecutive edits that share a merge key", () => {
    const scope = effectScope();
    try {
      scope.run(() => {
        const initial = fixture();
        const project = shallowRef(initial);
        const history = useVideoEditHistory(project);
        history.reset(initial);
        const clipId = initial.videoTrack.clips[0]!.id;
        history.commit(setClipVolume(initial, clipId, 0.5), "volume");
        history.commit(setClipVolume(project.value, clipId, 0.2), "volume");
        expect(project.value.videoTrack.clips[0]!.volume).toBe(0.2);
        history.undo();
        expect(project.value).toEqual(initial);
      });
    } finally {
      scope.stop();
    }
  });
});
