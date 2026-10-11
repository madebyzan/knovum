import { useElementSize } from "@vueuse/core";
import { computed, shallowRef, watch, watchPostEffect, type Ref } from "vue";

import { useSurfaceTriggerOpen } from "../use-surface-trigger-open";

export function useNodeActionBarDensity(
  bar: Readonly<Ref<HTMLElement | null>>,
  availableWidth: Readonly<Ref<number>>,
  contentIdentity: Readonly<Ref<string>>,
) {
  const { width } = useElementSize(bar);
  const transientOpen = useSurfaceTriggerOpen(bar);
  const expandedWidth = shallowRef(0);
  const compactWidth = shallowRef(0);
  const frozen = shallowRef<{ compact: boolean; minimal: boolean }>();
  const ready = computed(() => expandedWidth.value > 0);
  const compact = computed(
    () =>
      frozen.value?.compact ?? (expandedWidth.value > 0 && expandedWidth.value + 16 > availableWidth.value),
  );
  const minimal = computed(
    () => frozen.value?.minimal ?? (compact.value && compactWidth.value > availableWidth.value),
  );

  watch(
    transientOpen,
    (open) => {
      frozen.value = open ? { compact: compact.value, minimal: minimal.value } : undefined;
    },
    { flush: "sync" },
  );

  watch(
    contentIdentity,
    () => {
      expandedWidth.value = 0;
      compactWidth.value = 0;
      frozen.value = undefined;
    },
    { flush: "sync" },
  );
  watchPostEffect(() => {
    if (width.value <= 0) return;
    const measured = bar.value?.firstElementChild?.scrollWidth ?? width.value;
    if (compact.value) {
      if (!minimal.value) compactWidth.value = Math.max(compactWidth.value, measured);
    } else if (measured > 0) expandedWidth.value = Math.max(expandedWidth.value, measured);
  });

  return { compact, minimal, ready };
}
