import { useEventListener } from "@vueuse/core";
import { shallowRef, watch, type Ref } from "vue";

export type CanvasImageLoadState = "idle" | "loading" | "ready" | "failed";

export function useCanvasImageSource(
  source: Readonly<Ref<string | undefined>>,
  callbacks: { onReady: (image: HTMLImageElement) => void; onReset: () => void },
) {
  const image = new Image();
  image.crossOrigin = "anonymous";
  const naturalSize = shallowRef({ width: 0, height: 0 });
  const loadState = shallowRef<CanvasImageLoadState>("idle");

  function loaded() {
    if (!source.value) return;
    naturalSize.value = { width: image.naturalWidth, height: image.naturalHeight };
    loadState.value = "ready";
    callbacks.onReady(image);
  }
  function failed() {
    if (!source.value) return;
    loadState.value = "failed";
  }

  useEventListener(image, "load", loaded);
  useEventListener(image, "error", failed);
  watch(
    source,
    (value) => {
      naturalSize.value = { width: 0, height: 0 };
      loadState.value = value ? "loading" : "idle";
      callbacks.onReset();
      if (value) image.src = value;
      else image.removeAttribute("src");
    },
    { immediate: true },
  );

  return { image, naturalSize, loadState };
}
