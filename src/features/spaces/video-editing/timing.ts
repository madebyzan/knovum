import type {
  VideoAudioClip,
  VideoClip,
  VideoEditProject,
  VideoTextOverlay,
  VideoTransition,
} from "./project";

export function clipLengthMs(clip: VideoClip | VideoAudioClip) {
  return clip.sourceOutMs - clip.sourceInMs;
}

export function clipEndMs(clip: VideoClip | VideoAudioClip) {
  return clip.timelineStartMs + clipLengthMs(clip);
}

export function projectDurationMs(project: VideoEditProject) {
  return project.videoTrack.clips.reduce((end, clip) => Math.max(end, clipEndMs(clip)), 0);
}

export function clipIndexOf(project: VideoEditProject, clipId: string) {
  return project.videoTrack.clips.findIndex(({ id }) => id === clipId);
}

export function transitionFrom(project: VideoEditProject, clipId: string) {
  return project.transitions.find((transition) => transition.fromClipId === clipId);
}

function transitionInto(project: VideoEditProject, clipId: string) {
  const index = clipIndexOf(project, clipId);
  const previous = index > 0 ? project.videoTrack.clips[index - 1] : undefined;
  return previous ? transitionFrom(project, previous.id) : undefined;
}

/** Free room on each side of a clip before its neighbours or transitions are reached. */
export function clipHeadroom(project: VideoEditProject, clip: VideoClip) {
  return {
    incomingMs: transitionInto(project, clip.id)?.durationMs ?? 0,
    outgoingMs: transitionFrom(project, clip.id)?.durationMs ?? 0,
  };
}

function fadeGain(clip: VideoClip | VideoAudioClip, offsetMs: number) {
  return rampGain(clipLengthMs(clip), clip.fadeInMs, clip.fadeOutMs, offsetMs);
}

function rampGain(lengthMs: number, fadeInMs: number, fadeOutMs: number, offsetMs: number) {
  const remaining = lengthMs - offsetMs;
  return Math.max(0, Math.min(1, fadeInMs ? offsetMs / fadeInMs : 1, fadeOutMs ? remaining / fadeOutMs : 1));
}

/**
 * Audio gain of a visual clip. A transition overlap crossfades the sound, so the effective fade is
 * the longer of the clip's own fade and the overlap it shares with its neighbour — the same rule
 * the export applies.
 */
export function clipAudioGain(project: VideoEditProject, clip: VideoClip, offsetMs: number) {
  const { incomingMs, outgoingMs } = clipHeadroom(project, clip);
  return (
    clip.volume *
    rampGain(
      clipLengthMs(clip),
      Math.max(clip.fadeInMs, incomingMs),
      Math.max(clip.fadeOutMs, outgoingMs),
      offsetMs,
    )
  );
}

interface ActiveClip {
  clip: VideoClip;
  index: number;
  /** Position inside the clip's own media, relative to its source in-point. */
  offsetMs: number;
  sourceMs: number;
  fade: number;
}

export interface ActiveComposition {
  layers: ActiveClip[];
  transition?: { transition: VideoTransition; progress: number } | undefined;
}

/** Clips covering a timeline position, plus the transition weight when two of them overlap. */
export function compositionAt(project: VideoEditProject, timeMs: number): ActiveComposition {
  const layers = project.videoTrack.clips.flatMap((clip, index) => {
    if (timeMs < clip.timelineStartMs || timeMs >= clipEndMs(clip)) return [];
    const offsetMs = timeMs - clip.timelineStartMs;
    return [
      {
        clip,
        index,
        offsetMs,
        sourceMs: clip.sourceInMs + offsetMs,
        fade: fadeGain(clip, offsetMs),
      },
    ];
  });
  const outgoing = layers[0];
  const incoming = layers[1];
  if (!outgoing || !incoming) return { layers };
  const transition = transitionFrom(project, outgoing.clip.id);
  if (!transition) return { layers: [incoming] };
  const elapsed = timeMs - incoming.clip.timelineStartMs;
  return {
    layers,
    transition: { transition, progress: Math.min(1, Math.max(0, elapsed / transition.durationMs)) },
  };
}

export function textItemsAt(project: VideoEditProject, timeMs: number) {
  return project.textTrack.items.filter(
    (item: VideoTextOverlay) => timeMs >= item.startMs && timeMs < item.endMs,
  );
}

export function frameDurationMs(project: VideoEditProject) {
  return 1000 / project.output.frameRate;
}

export function snapToFrame(project: VideoEditProject, timeMs: number) {
  const frame = frameDurationMs(project);
  return Math.max(0, Math.round(timeMs / frame) * frame);
}

export function stepFrame(project: VideoEditProject, timeMs: number, direction: -1 | 1) {
  const frame = frameDurationMs(project);
  const index =
    direction === 1 ? Math.floor(timeMs / frame + 1e-6) + 1 : Math.ceil(timeMs / frame - 1e-6) - 1;
  return Math.max(0, Math.min(projectDurationMs(project), index * frame));
}

/** Last displayable position so the playhead can never rest past the final frame. */
export function lastFrameMs(project: VideoEditProject) {
  const frame = frameDurationMs(project);
  const frames = Math.ceil(projectDurationMs(project) / frame);
  return Math.max(0, (frames - 1) * frame);
}

export function canSplitAt(project: VideoEditProject, timeMs: number) {
  const { layers, transition } = compositionAt(project, Math.round(timeMs));
  const layer = layers[0];
  if (!layer || layers.length !== 1 || transition) return false;
  const { incomingMs, outgoingMs } = clipHeadroom(project, layer.clip);
  return (
    layer.offsetMs >= Math.max(incomingMs, layer.clip.fadeInMs) + 100 &&
    clipLengthMs(layer.clip) - layer.offsetMs >= Math.max(outgoingMs, layer.clip.fadeOutMs) + 100
  );
}
