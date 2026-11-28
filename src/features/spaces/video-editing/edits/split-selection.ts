import type { VideoEditProject } from "../project";
import { selectedAudio, selectedText, type VideoSelection } from "../selection";
import { canSplitAt } from "../timing";
import { splitClip } from "./arrange";
import { canSplitAudioAt, splitAudioClip } from "./audio";
import { canSplitTextAt, splitTextOverlayAt } from "./captions";

/** Checks whether the active selection (audio/text) or underlying video clip can be split at `atMs`. */
export function canSplitTarget(
  project: VideoEditProject,
  selection: VideoSelection | undefined,
  atMs: number,
): boolean {
  const audio = selectedAudio(project, selection);
  if (audio && canSplitAudioAt(audio, atMs)) return true;
  const text = selectedText(project, selection);
  if (text && canSplitTextAt(text, atMs)) return true;
  return canSplitAt(project, atMs);
}

/** Splits the selected audio/text block if the playhead lies inside it, otherwise splits the video clip. */
export function splitTargetAt(
  project: VideoEditProject,
  selection: VideoSelection | undefined,
  atMs: number,
): { project: VideoEditProject; nextSelection?: VideoSelection } {
  const at = Math.round(atMs);
  const audio = selectedAudio(project, selection);
  if (audio && canSplitAudioAt(audio, at)) {
    const next = splitAudioClip(project, audio.id, at);
    const tail = next.audioTrack.clips.find((c) => c.lane === audio.lane && c.timelineStartMs === at);
    return { project: next, ...(tail ? { nextSelection: { kind: "audio", id: tail.id } } : {}) };
  }
  const text = selectedText(project, selection);
  if (text && canSplitTextAt(text, at)) {
    const next = splitTextOverlayAt(project, text.id, at);
    const tail = next.textTrack.items.find((c) => c.startMs === at && c.id !== text.id);
    return { project: next, ...(tail ? { nextSelection: { kind: "text", id: tail.id } } : {}) };
  }
  const next = splitClip(project, at);
  if (next === project) return { project };
  const tail = next.videoTrack.clips.find((c) => c.timelineStartMs === at);
  return { project: next, ...(tail ? { nextSelection: { kind: "clip", id: tail.id } } : {}) };
}
