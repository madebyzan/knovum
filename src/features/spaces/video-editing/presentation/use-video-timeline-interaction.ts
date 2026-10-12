import { computed, shallowRef } from "vue";

import { clipTrimBounds } from "../clip-bounds";
import { replaceClip } from "../clip-edits";
import { type VideoClip, type VideoEditProject } from "../project";
import { clipDurationMs, clipTimelineStartMs } from "../timing";
import { useVideoTimelineScale } from "./use-video-timeline-scale";

export interface VideoTimelineProps {
  project: VideoEditProject;
  playheadMs: number;
  selectedClipId?: string | undefined;
  selectedTextId?: string | undefined;
  selectedAudioId?: string | undefined;
  zoom: number;
  playing: boolean;
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
  draft(project: VideoEditProject | undefined): void;
  pause(): void;
}

export function useVideoTimelineInteraction(props: VideoTimelineProps, actions: VideoTimelineActions) {
  const gesture = shallowRef<TrimGesture>();
  const draftClip = shallowRef<VideoClip>();
  const visibleProject = computed(() =>
    draftClip.value ? replaceClip(props.project, draftClip.value) : props.project,
  );
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
    actions.pause();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    seekFromPointer(event);
  }

  function moveScrub(event: PointerEvent) {
    if ((event.currentTarget as HTMLElement).hasPointerCapture(event.pointerId)) seekFromPointer(event);
  }

  function startTrim(event: PointerEvent, clip: VideoClip, edge: "start" | "end") {
    if (event.button !== 0) return;
    event.stopPropagation();
    actions.pause();
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
    const frame = 1000 / props.project.output.frameRate;
    const proposed = event.altKey
      ? Math.round(original + delta)
      : Math.round(Math.round((original + delta) / frame) * frame);
    const { minimum, maximum } = clipTrimBounds(props.project, current.clip);
    if (current.edge === "start") {
      draftClip.value = {
        ...current.clip,
        startMs: Math.max(0, Math.min(current.clip.endMs - minimum, proposed)),
      };
    } else {
      draftClip.value = {
        ...current.clip,
        endMs: Math.max(current.clip.startMs + minimum, Math.min(maximum, proposed)),
      };
    }
    actions.draft(visibleProject.value);
    const start = clipTimelineStartMs(visibleProject.value, current.clip.id);
    actions.seek(start + (current.edge === "start" ? 0 : clipDurationMs(draftClip.value!) - 1));
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
    const startMs = clipTimelineStartMs(props.project, edited.id);
    actions.trim(replaceClip(props.project, edited));
    actions.seek(startMs + (current.edge === "start" ? 0 : clipDurationMs(edited)));
  }

  function cancelTrim() {
    gesture.value = undefined;
    draftClip.value = undefined;
    actions.draft(undefined);
  }

  function nudgeTrim(clip: VideoClip, edge: "start" | "end", delta: number) {
    const { minimum, maximum } = clipTrimBounds(props.project, clip);
    const edited =
      edge === "start"
        ? { ...clip, startMs: Math.max(0, Math.min(clip.endMs - minimum, clip.startMs + delta)) }
        : { ...clip, endMs: Math.max(clip.startMs + minimum, Math.min(maximum, clip.endMs + delta)) };
    if (edited.startMs === clip.startMs && edited.endMs === clip.endMs) return;
    actions.pause();
    actions.selectClip(clip.id);
    actions.trim(replaceClip(props.project, edited));
  }
  function escape(event: KeyboardEvent) {
    if (!gesture.value) return;
    event.preventDefault();
    event.stopPropagation();
    cancelTrim();
  }

  return {
    escape,
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
