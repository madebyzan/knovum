import type { VideoClip, VideoEditProject } from "./project";

export function clipDurationMs(clip: Pick<VideoClip, "startMs" | "endMs">) {
  return clip.endMs - clip.startMs;
}
export function projectDurationMs(project: VideoEditProject) {
  return project.videoTrack.clips.reduce(
    (total, clip) => total + clipDurationMs(clip) - (clip.transition?.durationMs ?? 0),
    0,
  );
}
export function clipTimelineStartMs(project: VideoEditProject, clipId: string) {
  let cursorMs = 0;
  for (const clip of project.videoTrack.clips) {
    if (clip.id === clipId) return cursorMs;
    cursorMs += clipDurationMs(clip) - (clip.transition?.durationMs ?? 0);
  }
  throw new Error("Video clip is not in this project");
}
export function locateTimelineFrames(project: VideoEditProject, timelineMs: number) {
  let cursorMs = 0;
  const frames: {
    clip: VideoClip;
    clipIndex: number;
    offsetMs: number;
    timelineStartMs: number;
    progress: number;
  }[] = [];
  for (const [clipIndex, clip] of project.videoTrack.clips.entries()) {
    const durationMs = clipDurationMs(clip);
    if (timelineMs >= cursorMs && timelineMs < cursorMs + durationMs) {
      const incoming = project.videoTrack.clips[clipIndex - 1]?.transition?.durationMs ?? 0;
      frames.push({
        clip,
        clipIndex,
        offsetMs: timelineMs - cursorMs,
        timelineStartMs: cursorMs,
        progress: incoming ? Math.min(1, (timelineMs - cursorMs) / incoming) : 1,
      });
    }
    cursorMs += durationMs - (clip.transition?.durationMs ?? 0);
  }
  return frames;
}
export function locateTimelineTime(project: VideoEditProject, timelineMs: number) {
  return locateTimelineFrames(project, timelineMs).at(-1);
}
export function canSplitClip(project: VideoEditProject, timelineMs: number) {
  const frames = locateTimelineFrames(project, Math.round(timelineMs));
  const frame = frames[0];
  if (frames.length !== 1 || !frame) return false;
  const incoming = project.videoTrack.clips[frame.clipIndex - 1]?.transition?.durationMs ?? 0;
  const outgoing = frame.clip.transition?.durationMs ?? 0;
  return (
    frame.offsetMs >= Math.max(incoming + 100, frame.clip.fadeInMs) &&
    clipDurationMs(frame.clip) - frame.offsetMs >= Math.max(outgoing + 100, frame.clip.fadeOutMs)
  );
}
export function fadeGain(
  clip: { startMs: number; endMs: number; fadeInMs: number; fadeOutMs: number },
  offsetMs: number,
) {
  const remaining = clipDurationMs(clip) - offsetMs;
  return Math.min(
    1,
    clip.fadeInMs ? offsetMs / clip.fadeInMs : 1,
    clip.fadeOutMs ? remaining / clip.fadeOutMs : 1,
  );
}

export function stepOutputFrame(project: VideoEditProject, timeMs: number, direction: -1 | 1) {
  const frame = (timeMs * project.output.frameRate) / 1000;
  const index = direction === 1 ? Math.floor(frame + 1e-7) + 1 : Math.ceil(frame - 1e-7) - 1;
  return Math.max(0, Math.min(projectDurationMs(project), (index * 1000) / project.output.frameRate));
}
