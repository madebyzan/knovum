import { useDebounceFn, useThrottleFn } from "@vueuse/core";
import { onScopeDispose, shallowRef } from "vue";

import type { ImageCanvasSize } from "../../../image-editing/canvas-image-renderer";
import type { ImageEdit } from "../../../image-editing/image-edit";
import { useImageRenderWorker } from "../../../image-editing/use-image-render-worker";

export function useCanvasImageRaster(options: {
  image: HTMLImageElement;
  size: () => ImageCanvasSize;
  edit: () => ImageEdit;
  ready: () => boolean;
  active: () => boolean;
  paint: () => void;
  error: (error: unknown) => void;
}) {
  const worker = useImageRenderWorker();
  const owner = {};
  const bitmap = shallowRef<ImageBitmap>();
  let disposed = false;
  let revision = 0;
  const settle = useDebounceFn(() => request(false), 180);
  const preview = useThrottleFn(
    () => {
      if (disposed || !options.active() || !options.ready()) return;
      request(true);
      void settle();
    },
    100,
    true,
  );
  function update() {
    if (!options.ready()) return;
    if (options.active()) void preview();
    else {
      settle.cancel();
      request(false);
    }
  }
  function request(draft: boolean) {
    if (disposed || !options.ready()) return;
    const current = ++revision;
    const original = options.size();
    const ratio = Math.min(
      original.pixelRatio,
      Math.sqrt(2_000_000 / Math.max(1, original.width * original.height)),
      draft ? 512 / Math.max(original.width, original.height, 1) : Infinity,
    );
    const size = { ...original, pixelRatio: ratio };
    const currentEdit = options.edit();
    const edit: ImageEdit = {
      ...currentEdit,
      crop: { ...currentEdit.crop },
      adjustments: { ...currentEdit.adjustments },
    };
    void worker
      .render(
        owner,
        async () => ({ image: await createImageBitmap(options.image), size, edit }),
        options.active(),
      )
      .then((next) => {
        if (!next) return undefined;
        if (disposed || current !== revision) {
          next.close();
          return undefined;
        }
        bitmap.value?.close();
        bitmap.value = next;
        options.paint();
        return undefined;
      })
      .catch((error) => {
        if (!disposed && current === revision) options.error(error);
      });
  }
  function clear() {
    settle.cancel();
    revision++;
    worker.cancel(owner);
    bitmap.value?.close();
    bitmap.value = undefined;
  }
  onScopeDispose(() => {
    disposed = true;
    clear();
  });
  return { bitmap, clear, update };
}
