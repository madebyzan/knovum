import { defineStore, storeToRefs } from "pinia";

import { useSpaceStore } from "../session/space-store";
import { useSpaceConversation } from "./use-space-conversation";

export const useConversationSessionStore = defineStore("spaces-conversation-session", () => {
  const { activeSpaceId } = storeToRefs(useSpaceStore());
  return useSpaceConversation(activeSpaceId);
});
