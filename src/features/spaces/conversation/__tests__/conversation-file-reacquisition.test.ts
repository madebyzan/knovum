import { describe, expect, it } from "vitest";
import { ref } from "vue";

import { deferred } from "../../__tests__/support/canvas-fixtures";
import type { ConversationAttachment } from "../../contracts/generated/ConversationAttachment";
import { createConversationFilePicker } from "../conversation-file-reacquisition";

describe("conversation file reacquisition", () => {
  it("rejects attachment acquisition and recovery updates after A-B-A owner changes", async () => {
    const selection = deferred<void>();
    const spaceId = ref<string | undefined>("space-a");
    const conversationId = ref("conversation-a");
    const ownerEpoch = ref(1);
    const attachments = ref<ConversationAttachment[]>([]);
    const reacquireFileNames = ref(["shared.png"]);
    const pick = createConversationFilePicker({
      spaceId,
      conversationId,
      ownerEpoch: () => ownerEpoch.value,
      attachments,
      reacquireFileNames,
      choose: async (valid) => {
        await selection.promise;
        if (valid())
          attachments.value = [{ kind: "file", path: "/first/shared.png", sourceRevision: "first" }];
      },
    });
    const pending = pick();
    spaceId.value = "space-b";
    conversationId.value = "conversation-b";
    ownerEpoch.value += 1;
    spaceId.value = "space-a";
    conversationId.value = "conversation-a";
    ownerEpoch.value += 1;
    reacquireFileNames.value = ["shared.png"];
    selection.resolve();
    await pending;
    expect(attachments.value).toEqual([]);
    expect(reacquireFileNames.value).toEqual(["shared.png"]);
  });

  it("removes a recovery name only after its file is chosen for the current owner", async () => {
    const attachments = ref<ConversationAttachment[]>([]);
    const reacquireFileNames = ref(["chosen.png", "remaining.png"]);
    const ownerEpoch = ref(1);
    const pick = createConversationFilePicker({
      spaceId: ref("space"),
      conversationId: ref("conversation"),
      ownerEpoch: () => ownerEpoch.value,
      attachments,
      reacquireFileNames,
      choose: async (valid) => {
        expect(valid()).toBe(true);
        attachments.value.push({ kind: "file", path: "/picked/chosen.png", sourceRevision: "chosen" });
      },
    });
    await pick();
    expect(reacquireFileNames.value).toEqual(["remaining.png"]);
  });
});
