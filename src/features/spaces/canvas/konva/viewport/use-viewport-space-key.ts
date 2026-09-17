import { useEventListener } from "@vueuse/core";
import { shallowRef } from "vue";

import { isEditableTarget } from "../../../../../foundation/dom/editable-target";
import { isInteractiveTarget } from "../../../../../foundation/dom/interactive-target";

export function useViewportSpaceKey() {
  const spacePressed = shallowRef(false);

  function key(event: KeyboardEvent, pressed: boolean) {
    if (event.code !== "Space") return;
    if (pressed && (isEditableTarget(event.target) || isInteractiveTarget(event.target))) return;
    spacePressed.value = pressed;
    if (pressed) event.preventDefault();
  }

  useEventListener(globalThis, "keyup", (event: KeyboardEvent) => key(event, false));
  useEventListener(globalThis, "blur", () => {
    spacePressed.value = false;
  });

  return { key, spacePressed };
}
