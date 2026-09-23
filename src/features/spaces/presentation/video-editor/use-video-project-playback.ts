import { computed, type Ref, shallowRef, watch } from "vue";

import {
  clipDuration,
  locateTimelineTime,
  projectDuration,
  type VideoEditProject,
} from "../../video-editing/project";

export function useVideoProjectPlayback(video: Ref<HTMLVideoElement | null>, project: Ref<VideoEditProject>) {
  const timelineMs = shallowRef(0);
  const playing = shallowRef(false);
  const durationMs = computed(() => projectDuration(project.value));

  function seek(value: number) {
    timelineMs.value = Math.max(0, Math.min(value, durationMs.value));
    const location = locateTimelineTime(project.value, timelineMs.value);
    if (video.value && location) video.value.currentTime = (location.clip.startMs + location.offsetMs) / 1000;
  }

  async function toggle() {
    if (!video.value) return;
    if (playing.value) video.value.pause();
    else {
      if (timelineMs.value >= durationMs.value) seek(0);
      await video.value.play();
    }
  }

  function update() {
    const element = video.value;
    const location = locateTimelineTime(project.value, timelineMs.value);
    if (!element || !location) return;
    const sourceMs = element.currentTime * 1000;
    if (sourceMs >= location.clip.endMs - 20) {
      const next = project.value.videoTrack.clips[location.clipIndex + 1];
      if (!next) return element.pause();
      seek(timelineMs.value - location.offsetMs + clipDuration(location.clip));
      return;
    }
    timelineMs.value += Math.max(0, sourceMs - location.clip.startMs - location.offsetMs);
  }

  watch(video, (element, previous) => {
    previous?.removeEventListener("timeupdate", update);
    element?.addEventListener("timeupdate", update);
  });

  return { durationMs, playing, seek, timelineMs, toggle };
}
