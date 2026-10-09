import { basename } from "pathe";
import type { Ref } from "vue";

import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";

function remainingReacquisitions(
  names: readonly string[],
  attachments: readonly ConversationAttachment[],
  startIndex: number,
) {
  const reacquired = new Set(
    attachments
      .slice(startIndex)
      .filter((attachment) => attachment.kind === "file")
      .map((attachment) => basename(attachment.path)),
  );
  return names.filter((name) => !reacquired.has(name));
}

interface ConversationFilePickerInput {
  spaceId: Ref<string | undefined>;
  conversationId: Ref<string>;
  ownerEpoch: () => number;
  attachments: Ref<ConversationAttachment[]>;
  reacquireFileNames: Ref<string[]>;
  choose: (valid: () => boolean, paths?: readonly string[]) => Promise<void>;
}

export function createConversationFilePicker(input: ConversationFilePickerInput) {
  return async (paths?: readonly string[]) => {
    const selectedSpace = input.spaceId.value;
    const selectedConversation = input.conversationId.value;
    const selectedOwnerEpoch = input.ownerEpoch();
    const before = input.attachments.value.length;
    const isCurrentOwner = () =>
      input.ownerEpoch() === selectedOwnerEpoch &&
      input.spaceId.value === selectedSpace &&
      input.conversationId.value === selectedConversation;
    await input.choose(isCurrentOwner, paths);
    if (!isCurrentOwner()) return;
    input.reacquireFileNames.value = remainingReacquisitions(
      input.reacquireFileNames.value,
      input.attachments.value,
      before,
    );
  };
}
