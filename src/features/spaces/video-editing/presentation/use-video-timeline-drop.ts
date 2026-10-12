import { moveClipTo } from "../clip-order";
import type { VideoEditProject } from "../project";
export function videoTimelineDrop(
  event: DragEvent,
  project: VideoEditProject,
  seek: number,
  actions: { edit(project: VideoEditProject): void; add(nodeId: string, atMs: number): void },
) {
  const material = event.dataTransfer?.getData("application/x-knovum-video-material");
  if (material) {
    event.preventDefault();
    actions.add(material, seek);
    return;
  }
  const clip = event.dataTransfer?.getData("application/x-knovum-video-clip");
  if (!clip) return;
  event.preventDefault();
  const target = (event.target as HTMLElement).closest<HTMLElement>("[data-clip-id]")?.dataset.clipId;
  const index = project.videoTrack.clips.findIndex(({ id }) => id === target);
  if (index >= 0) actions.edit(moveClipTo(project, clip, index));
}
export function dragVideoClip(event: DragEvent, id: string) {
  event.dataTransfer?.setData("application/x-knovum-video-clip", id);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}
