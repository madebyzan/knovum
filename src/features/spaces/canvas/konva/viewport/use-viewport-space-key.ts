import { useEventListener } from "@vueuse/core";
import { shallowRef } from "vue";

import { isEditableTarget } from "../../../../../foundation/dom/editable-target";
import { isInteractiveTarget } from "../../../../../foundation/dom/interactive-target";

export function useViewportSpaceKey() {
  const spacePressed = shallowRef(false);

  function pressSpaceKey(event: KeyboardEvent) {
    if (event.code !== "Space") return;
    if (isEditableTarget(event.target) || isInteractiveTarget(event.target)) return;
    spacePressed.value = true;
    event.preventDefault();
  }

  function releaseSpaceKey(event: KeyboardEvent) {
    if (event.code !== "Space") return;
    spacePressed.value = false;
  }

  useEventListener(globalThis, "keyup", releaseSpaceKey);
  useEventListener(globalThis, "blur", () => {
    spacePressed.value = false;
  });

  return { pressSpaceKey, spacePressed };
}
