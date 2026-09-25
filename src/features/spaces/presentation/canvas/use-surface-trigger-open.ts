import { useMutationObserver } from "@vueuse/core";
import { readonly, shallowRef, watch, type Ref } from "vue";

/** Track expanded popup triggers without treating an open tooltip as an editing surface. */
export function useSurfaceTriggerOpen(element: Readonly<Ref<HTMLElement | null>>) {
  const open = shallowRef(false);
  const sync = () => (open.value = Boolean(element.value?.querySelector('[aria-expanded="true"]')));
  watch(element, sync, { immediate: true });
  useMutationObserver(element, sync, { attributes: true, subtree: true, attributeFilter: ["aria-expanded"] });
  return readonly(open);
}
