import { createGlobalState, useEventListener } from "@vueuse/core";
import { computed, shallowRef, watch } from "vue";

import type { TiptapJsonNode } from "../../../document-editing/tiptap-document";

interface TextMetrics {
  content: TiptapJsonNode;
  width: number;
  fontSize: number;
  scale: number;
  alignment: string;
}
interface Measurement extends Omit<TextMetrics, "content"> {
  fontEpoch: number;
  height: number;
}
const measurementByContent = new WeakMap<TiptapJsonNode, Measurement>();
const useFontEpoch = createGlobalState(() => {
  const epoch = shallowRef(0);
  useEventListener(document.fonts, "loadingdone", () => epoch.value++);
  return epoch;
});

export function useCanvasTextMeasurement(metrics: () => TextMetrics) {
  const fontEpoch = useFontEpoch();
  const cachedHeight = shallowRef<number>();
  watch(
    () => ({ ...metrics(), fontEpoch: fontEpoch.value }),
    (current) => {
      const cached = measurementByContent.get(current.content);
      cachedHeight.value = cached && sameMetrics(cached, current) ? cached.height : undefined;
    },
    { immediate: true },
  );
  return {
    cachedHeight,
    required: computed(() => cachedHeight.value === undefined),
    record(height: number) {
      const { content, ...current } = metrics();
      measurementByContent.set(content, { ...current, fontEpoch: fontEpoch.value, height });
      cachedHeight.value = height;
    },
  };
}

function sameMetrics(a: Measurement, b: Omit<Measurement, "height">) {
  return (
    a.width === b.width &&
    a.fontSize === b.fontSize &&
    a.scale === b.scale &&
    a.alignment === b.alignment &&
    a.fontEpoch === b.fontEpoch
  );
}
