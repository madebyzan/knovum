import { computed, provide, type Ref } from "vue";

import { SPEECH_EDITOR_VIEWPORT_KEY } from "./speech-editor-viewport";

/** Gives script controls the scrolling viewport owned by their editor. */
export function useSpeechEditorViewport(
  scroller: Readonly<Ref<{ viewportElement: HTMLElement | null | undefined } | null>>,
) {
  provide(
    SPEECH_EDITOR_VIEWPORT_KEY,
    computed(() => scroller.value?.viewportElement ?? null),
  );
}
