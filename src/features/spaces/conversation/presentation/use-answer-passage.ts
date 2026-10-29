import { writeText } from "tauri-plugin-clipboard-api";
import type { ShallowRef } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { usePointerDrag } from "../../../../shared/ui/use-pointer-drag";
import type { AnswerPassage } from "../answer-capture";
import type { useAnswerCapture } from "../use-answer-capture";
import { useAnswerSelection } from "./use-answer-selection";

type AnswerPassageAction = "save" | "append" | "followUp" | "copy";

/**
 * What a person does with words selected in an answer: the selection bar's actions, and dragging the
 * words onto the canvas. `quote` receives a passage to ask about.
 */
export function useAnswerPassage(
  body: Readonly<ShallowRef<HTMLElement | null>>,
  capture: ReturnType<typeof useAnswerCapture>,
  quote: (text: string) => void,
) {
  const selected = useAnswerSelection(body);

  // Each action ends the selection: the passage has been dealt with, and its bar would linger.
  async function act(action: AnswerPassageAction) {
    const passage = selected.passage.value;
    selected.clear();
    if (!passage) return;
    if (action === "save") await capture.saveSelection(passage);
    else if (action === "append") await capture.appendSelection(passage);
    else if (action === "followUp") quote(passage.text);
    else await writeText(passage.text).catch((error: unknown) => toast.error(getErrorMessage(error)));
  }

  const drag = usePointerDrag({
    targetAt: capture.canvasPointAt,
    // Clicking selected text ends the selection; the press is the drag's, so the tap does it.
    tap: selected.clear,
    drop: (passage: AnswerPassage, point) => {
      selected.clear();
      return capture.saveSelection(passage, point);
    },
  });

  // A press on the selected words picks them up; anywhere else it selects text as usual.
  function press(event: MouseEvent) {
    const passage = selected.passage.value;
    if (passage && selected.covers({ x: event.clientX, y: event.clientY })) drag.start(event, passage);
  }

  return { anchor: selected.anchor, act, press, drag: drag.drag };
}
