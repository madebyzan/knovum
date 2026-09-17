import { nextTick, watch, type Ref } from "vue";

const BOTTOM_THRESHOLD_PX = 56;

interface AssistantScrollState {
  viewport: Readonly<Ref<HTMLElement | null>>;
  busy: Readonly<Ref<boolean>>;
  pendingQuestion: Readonly<Ref<string>>;
  streamedAnswer: Readonly<Ref<string>>;
  messageCount: Readonly<Ref<number>>;
}

export function useAssistantScroll(state: AssistantScrollState) {
  let followTail = true;

  function atBottom(element: HTMLElement) {
    return element.scrollHeight - element.scrollTop - element.clientHeight <= BOTTOM_THRESHOLD_PX;
  }

  function onScroll() {
    const element = state.viewport.value;
    if (element) followTail = atBottom(element);
  }

  async function scrollToTail(force = false) {
    if (!force && !followTail) return;
    await nextTick();
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    const element = state.viewport.value;
    if (!element) return;
    element.scrollTop = element.scrollHeight;
  }

  watch(state.pendingQuestion, (question, previous) => {
    if (question && !previous) {
      followTail = true;
      void scrollToTail(true);
    }
  });
  watch(
    () => state.streamedAnswer.value.length,
    () => void scrollToTail(),
  );
  watch(state.messageCount, () => void scrollToTail());
  watch(state.busy, (active, previous) => {
    if (active && !previous) void scrollToTail(true);
    if (!active && previous) void scrollToTail();
  });

  return { onScroll };
}
