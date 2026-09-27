import { beforeEach, describe, expect, it, vi } from "vitest";
import { computed, shallowRef } from "vue";

import type { AssistantConversation, AssistantMessage } from "../conversation/conversation-ipc";
import { useConversationActions } from "../conversation/use-conversation-actions";

const ipc = vi.hoisted(() => ({ invoke: vi.fn<(...args: unknown[]) => Promise<void>>() }));
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: ipc.invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: vi.fn<() => Promise<boolean>>(),
  save: vi.fn<() => Promise<string | null>>(),
}));
vi.mock("@tauri-apps/plugin-fs", () => ({
  writeTextFile: vi.fn<(path: string, contents: string) => Promise<void>>(),
}));

function session() {
  const conversation: AssistantConversation = {
    id: "conversation",
    spaceId: "space",
    branchId: "branch",
    title: "Research",
    mode: "ask",
    updatedAt: 1,
  };
  const state = {
    selected: computed(() => conversation),
    selectedId: shallowRef(conversation.id),
    messages: shallowRef<AssistantMessage[]>([]),
    loading: shallowRef(false),
    prompt: shallowRef(""),
    refresh: vi.fn<() => Promise<void>>().mockResolvedValue(),
    select: vi.fn<() => Promise<void>>(),
    send: vi.fn<() => Promise<void>>(),
  };
  return { state, actions: useConversationActions(state) };
}

beforeEach(() => {
  ipc.invoke.mockReset().mockResolvedValue();
});

describe("conversation archival intent", () => {
  it("archives and clears the selection only after successful persistence", async () => {
    const { state, actions } = session();
    await actions.archiveSelected();
    expect(ipc.invoke).toHaveBeenCalledWith("plugin:spaces|archive_assistant_conversation", {
      conversationId: "conversation",
    });
    expect(state.selectedId.value).toBe("");
    expect(state.refresh).toHaveBeenCalledOnce();
  });
  it("restores an archived conversation using the opposite persistence intent", async () => {
    const { actions } = session();
    await actions.unarchiveSelected();
    expect(ipc.invoke).toHaveBeenCalledWith("plugin:spaces|unarchive_assistant_conversation", {
      conversationId: "conversation",
    });
  });
  it("retains the selection and propagates persistence failure", async () => {
    const { state, actions } = session();
    ipc.invoke.mockRejectedValueOnce(new Error("database unavailable"));
    await expect(actions.archiveSelected()).rejects.toThrow("database unavailable");
    expect(state.selectedId.value).toBe("conversation");
    expect(state.refresh).not.toHaveBeenCalled();
  });
});
