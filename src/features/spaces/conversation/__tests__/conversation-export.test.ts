import { beforeEach, describe, expect, it, vi } from "vitest";
import { computed, shallowRef } from "vue";

import type { Conversation } from "../../contracts/generated/Conversation";
import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import { createConversationActions } from "../conversation-actions";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args: unknown) => Promise<unknown>>(),
  save: vi.fn<(options: unknown) => Promise<string | null>>(),
  error: vi.fn<(message: string) => void>(),
}));
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: mocks.save }));
vi.mock("vue-sonner", () => ({ toast: { error: mocks.error, success: vi.fn<() => void>() } }));

function actions() {
  const conversation: Conversation = {
    id: "conversation",
    spaceId: "space",
    branchId: "branch",
    title: "Research / Notes",
    updatedAt: 1,
  };
  return createConversationActions({
    selected: computed(() => conversation),
    selectedId: shallowRef(conversation.id),
    messages: shallowRef<ConversationMessage[]>([]),
    loading: shallowRef(false),
    prompt: shallowRef(""),
    refresh: vi.fn<() => Promise<void>>(),
    select: vi.fn<() => Promise<void>>(),
    send: vi.fn<() => Promise<void>>(),
  });
}

beforeEach(() => {
  mocks.save.mockReset().mockResolvedValue("/exports/conversation.json");
  mocks.error.mockReset();
  mocks.invoke
    .mockReset()
    .mockImplementation(async (command) =>
      command === "plugin:spaces|load_conversation_branch_messages" ? [{ id: "message" }] : undefined,
    );
});

describe("conversation export", () => {
  it("writes through the authorized Rust export command", async () => {
    await actions().exportSelected();

    expect(mocks.save).toHaveBeenCalledWith({
      defaultPath: "Research - Notes.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|export_text", {
      destinationPath: "/exports/conversation.json",
      text: '[\n  {\n    "id": "message"\n  }\n]\n',
    });
  });

  it("reports a write failure without leaving an unhandled rejection", async () => {
    mocks.invoke.mockImplementation(async (command) => {
      if (command === "plugin:spaces|load_conversation_branch_messages") return [];
      throw new Error("disk full");
    });

    await expect(actions().exportSelected()).resolves.toBeUndefined();
    expect(mocks.error).toHaveBeenCalledWith(expect.stringContaining("disk full"));
  });
});
