import { computed, onBeforeUnmount, type Ref, shallowRef, watch } from "vue";

import { i18n } from "../../../../foundation/i18n/i18n";
import { locateTimelineTime, projectDuration, type VideoEditProject } from "../../video-editing/project";

export function useVideoProjectPlayback(
  video: Ref<HTMLVideoElement | null>,
  project: Ref<VideoEditProject | undefined>,
) {
  const timelineMs = shallowRef(0);
  const playing = shallowRef(false);
  const audioError = shallowRef<string>();
  const durationMs = computed(() => (project.value ? projectDuration(project.value) : 0));
  let frameRequest: number | undefined;
  let awaitingSeek = false;
  let audioContext: AudioContext | undefined;
  let audioSource: MediaElementAudioSourceNode | undefined;
  let gain: GainNode | undefined;

  function setGain(volume: number) {
    if (gain) gain.gain.value = volume;
  }

  function disconnectAudio() {
    audioSource?.disconnect();
    gain?.disconnect();
    if (audioContext) void audioContext.close();
    audioSource = undefined;
    gain = undefined;
    audioContext = undefined;
  }

  function seek(value: number) {
    if (!project.value) return;
    timelineMs.value = Math.max(0, Math.min(value, durationMs.value));
    const position = Math.min(timelineMs.value, Math.max(0, durationMs.value - 1));
    const location = locateTimelineTime(project.value, position);
    if (video.value && location) {
      setGain(location.clip.volume);
      const targetSeconds = (location.clip.startMs + location.offsetMs) / 1000;
      if (Math.abs(video.value.currentTime - targetSeconds) > 0.001) {
        awaitingSeek = true;
        video.value.currentTime = targetSeconds;
      }
    }
  }

  async function toggle() {
    if (!video.value || !project.value) return;
    if (playing.value) video.value.pause();
    else {
      if (timelineMs.value >= durationMs.value) seek(0);
      if (audioContext?.state === "suspended") await audioContext.resume();
      await video.value.play();
    }
  }

  function update(sourceMs: number) {
    const element = video.value;
    const currentProject = project.value;
    if (!element || !currentProject || awaitingSeek) return;
    const location = locateTimelineTime(currentProject, timelineMs.value);
    if (!location) return;
    setGain(location.clip.volume);
    if (sourceMs < location.clip.startMs - 20) return;
    if (sourceMs >= location.clip.endMs - 20) {
      const next = currentProject.videoTrack.clips[location.clipIndex + 1];
      if (!next) {
        timelineMs.value = durationMs.value;
        element.pause();
        return;
      }
      seek(location.timelineStartMs + (location.clip.endMs - location.clip.startMs));
      return;
    }
    timelineMs.value = location.timelineStartMs + Math.max(0, sourceMs - location.clip.startMs);
  }

  function onFrame(element: HTMLVideoElement, _now: number, metadata: VideoFrameCallbackMetadata) {
    if (video.value !== element) return;
    update(metadata.mediaTime * 1000);
    frameRequest = element.requestVideoFrameCallback((now, frame) => onFrame(element, now, frame));
  }

  watch(video, (element, previous) => {
    awaitingSeek = false;
    if (previous && frameRequest !== undefined) previous.cancelVideoFrameCallback(frameRequest);
    previous?.removeEventListener("seeked", onSeeked);
    previous?.removeEventListener("ended", onEnded);
    disconnectAudio();
    if (element) {
      try {
        audioContext = new AudioContext();
        audioSource = audioContext.createMediaElementSource(element);
        gain = audioContext.createGain();
        audioSource.connect(gain).connect(audioContext.destination);
        audioError.value = undefined;
      } catch (error) {
        disconnectAudio();
        audioError.value = `${i18n.global.t("spaces.media.audioPreviewUnavailable")}: ${String(error)}`;
      }
      element.addEventListener("seeked", onSeeked);
      element.addEventListener("ended", onEnded);
      frameRequest = element.requestVideoFrameCallback((now, frame) => onFrame(element, now, frame));
    }
  });
  onBeforeUnmount(() => {
    if (video.value && frameRequest !== undefined) video.value.cancelVideoFrameCallback(frameRequest);
    video.value?.removeEventListener("seeked", onSeeked);
    video.value?.removeEventListener("ended", onEnded);
    disconnectAudio();
  });

  function onSeeked() {
    awaitingSeek = false;
  }

  function onEnded() {
    if (!project.value || !video.value) return;
    const location = locateTimelineTime(project.value, Math.max(0, timelineMs.value - 1));
    if (location && project.value.videoTrack.clips[location.clipIndex + 1]) {
      seek(location.timelineStartMs + location.clip.endMs - location.clip.startMs);
      void video.value.play();
    } else {
      timelineMs.value = durationMs.value;
    }
  }

  function onSourceReady() {
    awaitingSeek = false;
    seek(timelineMs.value);
  }

  return { audioError, durationMs, onSourceReady, playing, seek, timelineMs, toggle };
}
