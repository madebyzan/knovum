import type { VideoEditProject } from "./project";
import { clipEndMs } from "./timing";

export type VideoSelection =
  | { kind: "clip"; id: string }
  | { kind: "text"; id: string }
  | { kind: "audio"; id: string }
  | { kind: "transition"; id: string };

/** Drops a selection whose target no longer exists after an edit. */
export function retainSelection(
  project: VideoEditProject,
  selection: VideoSelection | undefined,
): VideoSelection | undefined {
  if (!selection) return undefined;
  const exists = {
    clip: () => project.videoTrack.clips.some(({ id }) => id === selection.id),
    text: () => project.textTrack.items.some(({ id }) => id === selection.id),
    audio: () => project.audioTrack.clips.some(({ id }) => id === selection.id),
    transition: () => project.transitions.some(({ id }) => id === selection.id),
  };
  return exists[selection.kind]() ? selection : undefined;
}

export function selectedClip(project: VideoEditProject, selection: VideoSelection | undefined) {
  return selection?.kind === "clip"
    ? project.videoTrack.clips.find(({ id }) => id === selection.id)
    : undefined;
}

export function selectedText(project: VideoEditProject, selection: VideoSelection | undefined) {
  return selection?.kind === "text"
    ? project.textTrack.items.find(({ id }) => id === selection.id)
    : undefined;
}

export function selectedAudio(project: VideoEditProject, selection: VideoSelection | undefined) {
  return selection?.kind === "audio"
    ? project.audioTrack.clips.find(({ id }) => id === selection.id)
    : undefined;
}

export function selectedTransition(project: VideoEditProject, selection: VideoSelection | undefined) {
  return selection?.kind === "transition"
    ? project.transitions.find(({ id }) => id === selection.id)
    : undefined;
}

export interface SelectionSpan {
  startMs: number;
  endMs: number;
}

/** Timeline span a selection occupies, used to reveal it and to scope transport jumps. */
export function selectionSpan(
  project: VideoEditProject,
  selection: VideoSelection,
): SelectionSpan | undefined {
  const timed = selectedClip(project, selection) ?? selectedAudio(project, selection);
  if (timed) return { startMs: timed.timelineStartMs, endMs: clipEndMs(timed) };
  const text = selectedText(project, selection);
  if (text) return { startMs: text.startMs, endMs: text.endMs };
  return transitionSpan(project, selection);
}

function transitionSpan(project: VideoEditProject, selection: VideoSelection) {
  const transition = selectedTransition(project, selection);
  const from = transition && project.videoTrack.clips.find(({ id }) => id === transition.fromClipId);
  if (!from || !transition) return undefined;
  const endMs = clipEndMs(from);
  return { startMs: endMs - transition.durationMs, endMs };
}
