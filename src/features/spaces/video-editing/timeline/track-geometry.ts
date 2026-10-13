import type { VideoEditProject } from "../project";

/**
 * One source of truth for timeline row heights. Track headers and lanes read the same numbers, so
 * they cannot drift apart.
 */
export const TIMELINE_ROWS = {
  ruler: 28,
  text: 30,
  video: 72,
  audio: 30,
} as const;

export const AUDIO_LANE_LIMIT = 4;

function audioLaneCount(project: VideoEditProject) {
  const used = project.audioTrack.clips.reduce((lanes, clip) => Math.max(lanes, clip.lane + 1), 1);
  return Math.min(AUDIO_LANE_LIMIT, used);
}

export function rowOffsets(project: VideoEditProject) {
  const text = TIMELINE_ROWS.ruler;
  const video = text + TIMELINE_ROWS.text;
  const audio = video + TIMELINE_ROWS.video;
  return {
    text,
    video,
    audio,
    lane: (index: number) => audio + index * TIMELINE_ROWS.audio,
    lanes: audioLaneCount(project),
  };
}
