import { keepReferencedVideoAssets } from "./material";
import type { VideoEditProject } from "./project";
import { projectDurationMs } from "./timing";

interface TimelineSplice {
  atMs: number;
  removedMs: number;
  insertedMs: number;
}

export function spliceTimeline(project: VideoEditProject, splices: TimelineSplice[]): VideoEditProject {
  const duration = projectDurationMs(project);
  const mapTime = (value: number) =>
    splices.reduce((time, { atMs, removedMs, insertedMs }) => {
      if (time < atMs) return time;
      if (removedMs > 0 && time <= atMs + removedMs) return atMs;
      return time - removedMs + insertedMs;
    }, value);
  return keepReferencedVideoAssets({
    ...project,
    textTrack: {
      items: project.textTrack.items
        .map((item) => ({
          ...item,
          startMs: mapTime(item.startMs),
          endMs: Math.min(duration, mapTime(item.endMs)),
        }))
        .filter((item) => item.endMs > item.startMs),
    },
    audioTrack: {
      clips: project.audioTrack.clips
        .map((clip) => {
          const timelineStartMs = mapTime(clip.timelineStartMs);
          const endMs = Math.min(clip.endMs, clip.startMs + duration - timelineStartMs);
          const length = Math.max(0, endMs - clip.startMs);
          return {
            ...clip,
            timelineStartMs,
            endMs,
            fadeInMs: Math.min(clip.fadeInMs, Math.floor(length / 2)),
            fadeOutMs: Math.min(clip.fadeOutMs, Math.floor(length / 2)),
          };
        })
        .filter((clip) => clip.endMs - clip.startMs >= 100),
    },
  });
}
