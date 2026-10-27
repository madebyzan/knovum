import { beforeEach, describe, expect, it, vi } from "vitest";
import { computed, shallowRef } from "vue";

import type { Conversation } from "../../contracts/generated/Conversation";
import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import { createConversationActions } from "../conversation-actions";
import type { ConversationSelectionOptions } from "../conversation-selection";

const ipc = vi.hoisted(() => ({ invoke: vi.fn<(...args: unknown[]) => Promise<void>>() }));
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: ipc.invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: vi.fn<() => Promise<null>>() }));
vi.mock("../../../../shared/ui/confirm-action", () => ({ confirmAction: async () => true }));

function session() {
  const conversation: Conversation = {
    id: "conversation",
    spaceId: "space",
    branchId: "branch",
    title: "Research",
    updatedAt: 1,
  };
  const state = {
    selected: computed(() => conversation),
    messages: shallowRef<ConversationMessage[]>([]),
    loading: shallowRef(false),
    prompt: shallowRef(""),
    refresh: vi.fn<() => Promise<void>>().mockResolvedValue(),
    select: vi.fn<(conversationId: string, options?: ConversationSelectionOptions) => Promise<void>>(),
    send: vi.fn<() => Promise<void>>(),
  };
  return { state, actions: createConversationActions(state) };
}

beforeEach(() => {
  ipc.invoke.mockReset().mockResolvedValue();
});

describe("conversation archival intent", () => {
  it("archives, then leaves through selection so the draft stays with its Conversation", async () => {
    const { state, actions } = session();
    await actions.archiveSelected();
    expect(ipc.invoke).toHaveBeenCalledWith("plugin:spaces|archive_conversation", {
      conversationId: "conversation",
    });
    expect(state.select).toHaveBeenCalledExactlyOnceWith("");
    expect(state.refresh).toHaveBeenCalledOnce();
  });
  it("restores an archived conversation using the opposite persistence intent", async () => {
    const { actions } = session();
    await actions.unarchiveSelected();
    expect(ipc.invoke).toHaveBeenCalledWith("plugin:spaces|unarchive_conversation", {
      conversationId: "conversation",
    });
  });
  it("retains the selection and propagates persistence failure", async () => {
    const { state, actions } = session();
    ipc.invoke.mockRejectedValueOnce(new Error("database unavailable"));
    await expect(actions.archiveSelected()).rejects.toThrow("database unavailable");
    expect(state.select).not.toHaveBeenCalled();
    expect(state.refresh).not.toHaveBeenCalled();
  });
  it("leaves a deleted Conversation without saving its draft into it", async () => {
    const { state, actions } = session();
    await actions.deleteSelected();
    expect(ipc.invoke).toHaveBeenCalledWith("plugin:spaces|delete_conversation", {
      conversationId: "conversation",
    });
    expect(state.select).toHaveBeenCalledExactlyOnceWith("", { rememberDraft: false });
  });
});
