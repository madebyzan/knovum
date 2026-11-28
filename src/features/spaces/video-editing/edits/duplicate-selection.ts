import type { VideoEditProject } from "../project";
import type { VideoSelection } from "../selection";
import { duplicateClip } from "./arrange";
import { duplicateAudioClip } from "./audio";
import { duplicateTextOverlay } from "./text";

export function duplicateSelectedItem(
  project: VideoEditProject,
  selection: VideoSelection | undefined,
): { project: VideoEditProject; nextSelection?: VideoSelection } {
  if (!selection || selection.kind === "transition") return { project };
  if (selection.kind === "clip") {
    return { project: duplicateClip(project, selection.id) };
  }
  if (selection.kind === "audio") {
    const next = duplicateAudioClip(project, selection.id);
    if (next === project) return { project };
    const added = next.audioTrack.clips.find((c) => !project.audioTrack.clips.some((o) => o.id === c.id));
    return { project: next, ...(added ? { nextSelection: { kind: "audio", id: added.id } } : {}) };
  }
  const next = duplicateTextOverlay(project, selection.id);
  if (next === project) return { project };
  const added = next.textTrack.items.at(-1);
  return { project: next, ...(added ? { nextSelection: { kind: "text", id: added.id } } : {}) };
}
