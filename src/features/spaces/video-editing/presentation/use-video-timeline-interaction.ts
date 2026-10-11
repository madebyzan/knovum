import { sumBy } from "es-toolkit";
import { computed, shallowRef } from "vue";

import { clipDurationMs, replaceClip, type VideoClip, type VideoEditProject } from "../project";
import { useVideoTimelineScale } from "./use-video-timeline-scale";

export interface VideoTimelineProps {
  project: VideoEditProject;
  sourceDurationMs: number;
  playheadMs: number;
  selectedClipId?: string | undefined;
  selectedTextId?: string | undefined;
  zoom: number;
  playing: boolean;
  thumbnailSource?: string | undefined;
  waveform?: number[] | undefined;
}

interface TrimGesture {
  clip: VideoClip;
  edge: "start" | "end";
  originX: number;
  trackWidth: number;
  timelineDurationMs: number;
}

interface VideoTimelineActions {
  seek(milliseconds: number): void;
  trim(project: VideoEditProject): void;
  selectClip(clipId: string): void;
}

export function useVideoTimelineInteraction(props: VideoTimelineProps, actions: VideoTimelineActions) {
  const gesture = shallowRef<TrimGesture>();
  const draftClip = shallowRef<VideoClip>();
  const visibleProject = computed(() => {
    const draft = draftClip.value;
    return draft
      ? {
          ...props.project,
          videoTrack: {
            clips: props.project.videoTrack.clips.map((clip) => (clip.id === draft.id ? draft : clip)),
          },
        }
      : props.project;
  });
  const { viewport, durationMs, ticks, contentStyle, clipStyle } = useVideoTimelineScale({
    project: visibleProject,
    zoom: () => props.zoom,
    playheadMs: () => props.playheadMs,
    playing: () => props.playing,
  });

  function seekFromPointer(event: PointerEvent) {
    const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
    actions.seek(
      Math.max(
        0,
        Math.min(durationMs.value, ((event.clientX - bounds.left) / bounds.width) * durationMs.value),
      ),
    );
  }

  function startScrub(event: PointerEvent) {
    if (event.button !== 0) return;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    seekFromPointer(event);
  }

  function moveScrub(event: PointerEvent) {
    if ((event.currentTarget as HTMLElement).hasPointerCapture(event.pointerId)) seekFromPointer(event);
  }

  function startTrim(event: PointerEvent, clip: VideoClip, edge: "start" | "end") {
    if (event.button !== 0) return;
    event.stopPropagation();
    const target = event.currentTarget as HTMLElement;
    const trackWidth = target.closest<HTMLElement>("[data-video-track]")?.getBoundingClientRect().width;
    if (!trackWidth) return;
    target.setPointerCapture(event.pointerId);
    gesture.value = { clip, edge, originX: event.clientX, trackWidth, timelineDurationMs: durationMs.value };
    draftClip.value = clip;
    actions.selectClip(clip.id);
  }

  function moveTrim(event: PointerEvent) {
    const current = gesture.value;
    if (!current) return;
    const delta = ((event.clientX - current.originX) / current.trackWidth) * current.timelineDurationMs;
    const original = current.edge === "start" ? current.clip.startMs : current.clip.endMs;
    const proposed = event.altKey ? Math.round(original + delta) : Math.round((original + delta) / 100) * 100;
    if (current.edge === "start") {
      draftClip.value = {
        ...current.clip,
        startMs: Math.max(0, Math.min(current.clip.endMs - 100, proposed)),
      };
    } else {
      const maximum = Math.max(props.sourceDurationMs, current.clip.endMs);
      draftClip.value = {
        ...current.clip,
        endMs: Math.max(current.clip.startMs + 100, Math.min(maximum, proposed)),
      };
    }
  }

  function finishTrim() {
    const current = gesture.value;
    const edited = draftClip.value;
    cancelTrim();
    if (
      !current ||
      !edited ||
      (edited.startMs === current.clip.startMs && edited.endMs === current.clip.endMs)
    )
      return;
    const index = props.project.videoTrack.clips.findIndex(({ id }) => id === edited.id);
    if (index < 0) return;
    const startMs = sumBy(props.project.videoTrack.clips.slice(0, index), clipDurationMs);
    actions.trim(replaceClip(props.project, edited));
    actions.seek(startMs + (current.edge === "start" ? 0 : clipDurationMs(edited)));
  }

  function cancelTrim() {
    gesture.value = undefined;
    draftClip.value = undefined;
  }

  function nudgeTrim(clip: VideoClip, edge: "start" | "end", delta: number) {
    const edited =
      edge === "start"
        ? { ...clip, startMs: Math.max(0, Math.min(clip.endMs - 100, clip.startMs + delta)) }
        : {
            ...clip,
            endMs: Math.max(
              clip.startMs + 100,
              Math.min(Math.max(props.sourceDurationMs, clip.endMs), clip.endMs + delta),
            ),
          };
    if (edited.startMs === clip.startMs && edited.endMs === clip.endMs) return;
    actions.selectClip(clip.id);
    actions.trim(replaceClip(props.project, edited));
  }

  return {
    visibleProject,
    viewport,
    durationMs,
    ticks,
    contentStyle,
    clipStyle,
    startScrub,
    moveScrub,
    startTrim,
    moveTrim,
    finishTrim,
    cancelTrim,
    nudgeTrim,
  };
}
