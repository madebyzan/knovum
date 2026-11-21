import { computed, shallowRef } from "vue";

const FONT_SCALES = [0.9, 1, 1.1, 1.2, 1.3] as const;

/** Shared comfortable typography scale (`A-` / `A+`) across long-form reading surfaces. */
export function useReaderTypography() {
  const stepIndex = shallowRef(1);
  const scale = computed(() => FONT_SCALES[stepIndex.value] ?? 1);
  const canDecrease = computed(() => stepIndex.value > 0);
  const canIncrease = computed(() => stepIndex.value < FONT_SCALES.length - 1);
  const proseStyle = computed(() => ({ "--reader-font-scale": String(scale.value) }));

  function decrease() {
    if (canDecrease.value) stepIndex.value -= 1;
  }

  function increase() {
    if (canIncrease.value) stepIndex.value += 1;
  }

  return { scale, canDecrease, canIncrease, proseStyle, decrease, increase };
}
