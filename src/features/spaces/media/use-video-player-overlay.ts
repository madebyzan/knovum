import { onScopeDispose, shallowRef, watch, type Ref } from "vue";

const AUTO_HIDE_DELAY_MS = 2000;

export function useVideoPlayerOverlay(playing: Ref<boolean>) {
  const controlsVisible = shallowRef(true);
  const flashFeedback = shallowRef<"play" | "pause">();

  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let flashTimer: ReturnType<typeof setTimeout> | undefined;

  function clearHideTimer() {
    if (hideTimer) {
      clearTimeout(hideTimer);
      hideTimer = undefined;
    }
  }

  function scheduleAutoHide() {
    clearHideTimer();
    if (!playing.value) return;
    hideTimer = setTimeout(() => {
      controlsVisible.value = false;
    }, AUTO_HIDE_DELAY_MS);
  }

  function wakeControls() {
    controlsVisible.value = true;
    scheduleAutoHide();
  }

  function handlePointerLeave() {
    clearHideTimer();
    if (playing.value) controlsVisible.value = false;
  }

  function triggerFlash(action: "play" | "pause") {
    if (flashTimer) clearTimeout(flashTimer);
    flashFeedback.value = action;
    flashTimer = setTimeout(() => {
      flashFeedback.value = undefined;
    }, 450);
  }

  watch(playing, (isPlaying) => {
    if (isPlaying) {
      scheduleAutoHide();
    } else {
      clearHideTimer();
      controlsVisible.value = true;
    }
  });

  onScopeDispose(() => {
    clearHideTimer();
    if (flashTimer) clearTimeout(flashTimer);
  });

  return {
    controlsVisible,
    flashFeedback,
    handlePointerLeave,
    triggerFlash,
    wakeControls,
  };
}
