import { watch, type Ref } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";

export function useConversationSpaceRestoration(input: {
  spaceId: Ref<string | undefined>;
  error: Ref<string | undefined>;
  reset: () => void;
  restoreDraft: (spaceId?: string) => Promise<boolean>;
  refresh: () => Promise<void>;
  restoreTask: (spaceId: string) => Promise<void>;
}) {
  let generation = 0;

  function isCurrent(requested: number, spaceId: string | undefined) {
    return generation === requested && input.spaceId.value === spaceId;
  }

  async function restoreSpace(spaceId: string | undefined) {
    const requested = ++generation;
    input.reset();
    try {
      await input.restoreDraft(spaceId);
      if (!isCurrent(requested, spaceId)) return;
      await input.refresh();
      if (!isCurrent(requested, spaceId)) return;
      if (spaceId) await input.restoreTask(spaceId);
      if (isCurrent(requested, spaceId)) input.error.value = undefined;
    } catch (error) {
      if (isCurrent(requested, spaceId)) input.error.value = getErrorMessage(error);
    }
  }

  watch(input.spaceId, (spaceId) => void restoreSpace(spaceId), { immediate: true });
}
