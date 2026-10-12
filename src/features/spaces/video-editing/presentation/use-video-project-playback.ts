import { useRafFn } from "@vueuse/core";
import { computed, shallowRef, watch, type Ref } from "vue";

import type { VideoEditProject } from "../project";
import { projectDurationMs } from "../timing";

export function useVideoProjectPlayback(
  project: Ref<VideoEditProject | undefined>,
  active: Ref<boolean>,
  timelineMs = shallowRef(0),
) {
  const playing = shallowRef(false);
  const ready = shallowRef(false);
  const durationMs = computed(() => (project.value ? projectDurationMs(project.value) : 0));
  function pause() {
    playing.value = false;
  }
  function seek(value: number) {
    timelineMs.value = Math.max(0, Math.min(value, durationMs.value));
  }
  function toggle() {
    if (!project.value || !active.value) return;
    if (timelineMs.value >= durationMs.value) seek(0);
    playing.value = !playing.value;
  }
  const clock = useRafFn(
    ({ delta }) => {
      if (!playing.value || !ready.value) return;
      seek(timelineMs.value + delta);
      if (timelineMs.value >= durationMs.value) pause();
    },
    { immediate: false },
  );
  watch(playing, (value) => (value ? clock.resume() : clock.pause()));
  watch(active, (value) => {
    if (!value) pause();
  });
  watch(durationMs, () => seek(timelineMs.value));
  return { playing, ready, durationMs, timelineMs, seek, toggle, pause };
}
