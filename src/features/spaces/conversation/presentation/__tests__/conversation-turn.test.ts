import { mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import { describe, expect, it } from "vitest";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { RunEvent } from "../../../contracts/generated/RunEvent";
import { conversationMessage, runEvent } from "../../__tests__/support/conversation-transcripts";
import type { LiveTranscript } from "../../conversation-transcript";
import type { ConversationTurn } from "../../conversation-turns";
import SpaceConversationActivity from "../SpaceConversationActivity.vue";
import SpaceConversationShimmerText from "../SpaceConversationShimmerText.vue";
import SpaceConversationTurn from "../SpaceConversationTurn.vue";

const translate = (key: string, params?: Record<string, unknown>) =>
  params?.count !== undefined ? `${key}:${params.count}` : key;

function createTurn(overrides: Partial<ConversationTurn> = {}): ConversationTurn {
  return {
    id: "ask-1",
    prompt: conversationMessage("ask-1", "user", "Summarize this note"),
    answers: [],
    runId: "run",
    active: true,
    live: [],
    ...overrides,
  };
}

const started = (id: string, sequence: number) =>
  runEvent("conversation_tool_started", { toolCallId: id, title: `Read ${id}` }, sequence);
const finished = (id: string, sequence: number) =>
  runEvent("conversation_tool_finished", { toolCallId: id, status: "completed" }, sequence);

function live(...steps: LiveTranscript) {
  return createTurn({ live: steps });
}

function mountTurn(turn: ConversationTurn, events: readonly RunEvent[] = []) {
  return mount(SpaceConversationTurn, {
    props: { turn, messages: turn.prompt ? [turn.prompt] : [], events, busy: true },
    global: {
      plugins: [createPinia(), i18n],
      mocks: { $t: translate },
      stubs: {
        SpaceConversationMessage: true,
        UiMarkdown: {
          name: "UiMarkdown",
          props: ["content"],
          template: '<div data-test="markdown">{{ content }}</div>',
        },
      },
    },
  });
}

function blocks(wrapper: ReturnType<typeof mountTurn>) {
  return wrapper
    .findAll('[data-test="markdown"], details')
    .map((element) => (element.element.tagName === "DETAILS" ? "tools" : element.text()));
}

describe("SpaceConversationTurn", () => {
  it("shows one waiting line until the run has something to show", async () => {
    const wrapper = mountTurn(createTurn({ runId: undefined }));
    try {
      const shimmer = wrapper.findComponent(SpaceConversationShimmerText);
      expect(shimmer.props("indicator")).toBe(true);
      expect(shimmer.findAll(".conversation-work-mark")).toHaveLength(3);
      expect(shimmer.text()).toBe("spaces.conversation.conversations.preparing");

      await wrapper.setProps({ turn: createTurn() });
      expect(wrapper.findComponent(SpaceConversationShimmerText).text()).toBe(
        "spaces.conversation.activity.generating",
      );

      await wrapper.setProps({ turn: createTurn({ live: [{ kind: "text", text: "Here" }] }) });
      expect(wrapper.text()).not.toContain("spaces.conversation.activity.generating");
    } finally {
      wrapper.unmount();
    }
  });

  it("keeps tool calls folded where they happened, and never opens or closes them by itself", async () => {
    const wrapper = mountTurn(live({ kind: "text", text: "Looking." }, { kind: "tool", id: "a" }), [
      started("a", 1),
    ]);
    try {
      const details = () => wrapper.find("details");
      expect(blocks(wrapper)).toEqual(["Looking.", "tools"]);
      expect(details().attributes("open")).toBeUndefined();
      expect(wrapper.findComponent(SpaceConversationActivity).text()).toContain(
        "spaces.conversation.activity.running:1",
      );

      // Between two calls the group still says the run is working there; no second line appears
      // below it only to vanish when the next call begins.
      await wrapper.setProps({ events: [started("a", 1), finished("a", 2)] });
      expect(wrapper.text()).not.toContain("spaces.conversation.activity.generating");
      expect(wrapper.findComponent(SpaceConversationActivity).text()).toContain(
        "spaces.conversation.activity.running:1",
      );

      // Opened by the person, it stays open as the run goes on.
      (details().element as HTMLDetailsElement).open = true;
      await wrapper.setProps({
        turn: live({ kind: "text", text: "Looking." }, { kind: "tool", id: "a" }, { kind: "tool", id: "b" }),
        events: [started("a", 1), finished("a", 2), started("b", 3)],
      });
      expect((details().element as HTMLDetailsElement).open).toBe(true);
      expect(wrapper.findComponent(SpaceConversationActivity).text()).toContain(
        "spaces.conversation.activity.running:2",
      );

      // Words after the calls come after them; the group above settles and keeps its state.
      await wrapper.setProps({
        turn: live(
          { kind: "text", text: "Looking." },
          { kind: "tool", id: "a" },
          { kind: "tool", id: "b" },
          { kind: "text", text: "Found it." },
        ),
        events: [started("a", 1), finished("a", 2), started("b", 3), finished("b", 4)],
      });
      expect(blocks(wrapper)).toEqual(["Looking.", "tools", "Found it."]);
      expect((details().element as HTMLDetailsElement).open).toBe(true);
      expect(wrapper.findComponent(SpaceConversationActivity).text()).toContain(
        "spaces.conversation.activity.finished:2",
      );
    } finally {
      wrapper.unmount();
    }
  });
});
