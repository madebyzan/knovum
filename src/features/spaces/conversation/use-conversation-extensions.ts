import { useQuery } from "@tanstack/vue-query";
import { cloneDeep } from "es-toolkit";
import { computed, shallowRef, watch, type Ref } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { extensionAvailabilityQuery } from "../../../foundation/extensions/extension-query";
import type { AgentExtensionSelection } from "../../../shared/contracts/generated/AgentExtensionSelection";
import { useConversationDraftStore } from "./conversation-draft-store";

export function useConversationExtensions(spaceId: Ref<string | undefined>, conversationId: Ref<string>) {
  const drafts = useConversationDraftStore();
  const availability = useQuery(extensionAvailabilityQuery);
  const selection = shallowRef<AgentExtensionSelection>({ skillIds: [], connections: [] });
  const key = computed(() => `${spaceId.value ?? ""}:${conversationId.value}`);
  let initialized = "";
  watch(
    [key, () => availability.data.value],
    ([selected, available]) => {
      if (!available || initialized === selected) return;
      if (!drafts.read(spaceId.value ?? "", conversationId.value))
        selection.value = cloneDeep(available.defaultSelection);
      initialized = selected;
    },
    { immediate: true },
  );
  return {
    extensionSelection: selection,
    extensionAvailability: availability.data,
    extensionsLoading: availability.isPending,
    extensionsError: computed(() =>
      availability.error.value ? getErrorMessage(availability.error.value) : undefined,
    ),
  };
}
