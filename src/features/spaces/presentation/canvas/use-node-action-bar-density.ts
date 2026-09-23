import { useElementSize } from "@vueuse/core";
import { computed, shallowRef, watch, watchEffect, type Ref } from "vue";

export function useNodeActionBarDensity(
  bar: Readonly<Ref<HTMLElement | null>>,
  availableWidth: Readonly<Ref<number>>,
  contentIdentity: Readonly<Ref<string>>,
) {
  const { width } = useElementSize(bar);
  const expandedWidth = shallowRef(0);
  const compact = computed(() => expandedWidth.value > 0 && expandedWidth.value + 16 > availableWidth.value);

  watch(
    contentIdentity,
    () => {
      expandedWidth.value = 0;
    },
    { flush: "sync" },
  );
  watchEffect(() => {
    if (compact.value || width.value <= 0) return;
    const measured = bar.value?.firstElementChild?.scrollWidth ?? width.value;
    if (measured > 0) expandedWidth.value = Math.max(expandedWidth.value, measured);
  });

  return { compact };
}
