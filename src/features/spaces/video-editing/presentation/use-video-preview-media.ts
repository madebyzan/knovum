import { useEventListener } from "@vueuse/core";
import { onScopeDispose, watch, type Ref } from "vue";

interface PreviewMedia {
  element: Ref<HTMLMediaElement | null>;
  time: () => number;
  gain: () => number;
  playing: () => boolean;
  failed(error: unknown): void;
}
export function useVideoPreviewMedia(options: PreviewMedia) {
  let context: AudioContext | undefined;
  let gain: GainNode | undefined;
  let pending = false;
  let disposed = false;
  function disconnect() {
    gain?.disconnect();
    if (context) void context.close();
    context = undefined;
    gain = undefined;
  }
  watch(
    options.element,
    (element, previous) => {
      previous?.pause();
      disconnect();
      pending = false;
      if (!element) return;
      try {
        context = new AudioContext();
        gain = context.createGain();
        context.createMediaElementSource(element).connect(gain).connect(context.destination);
      } catch (error) {
        options.failed(error);
      }
    },
    { flush: "sync" },
  );
  function sync() {
    const element = options.element.value;
    if (!element || element.readyState < 1) return;
    if (gain) gain.gain.value = Math.max(0, options.gain());
    const target = options.time();
    if (Math.abs(element.currentTime - target) > (options.playing() ? 0.08 : 0.001))
      element.currentTime = target;
    if (!options.playing()) {
      element.pause();
      return;
    }
    if (!element.paused || pending) return;
    pending = true;
    void play(element)
      .catch((error) => {
        if (!disposed && options.element.value === element && options.playing()) options.failed(error);
      })
      .finally(() => {
        pending = false;
      });
  }
  async function play(element: HTMLMediaElement) {
    if (context?.state === "suspended") await context.resume();
    if (disposed || options.element.value !== element || !options.playing()) return;
    await element.play();
    if (disposed || !options.playing() || options.element.value !== element) element.pause();
  }
  watch([options.element, options.time, options.gain, options.playing], sync);
  useEventListener(options.element, "loadedmetadata", sync);
  useEventListener(options.element, "error", () => options.failed(new Error("Media preview is unavailable")));
  onScopeDispose(() => {
    disposed = true;
    options.element.value?.pause();
    disconnect();
  });
}
