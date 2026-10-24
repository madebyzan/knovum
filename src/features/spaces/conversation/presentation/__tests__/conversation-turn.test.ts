import { mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import { describe, expect, it } from "vitest";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { RunEvent } from "../../../contracts/generated/RunEvent";
import type { ConversationTurn } from "../../conversation-turns";
import SpaceConversationActivity from "../SpaceConversationActivity.vue";
import SpaceConversationShimmerText from "../SpaceConversationShimmerText.vue";
import SpaceConversationTurn from "../SpaceConversationTurn.vue";

const translate = (key: string, params?: Record<string, unknown>) =>
  params?.count !== undefined ? `${key}:${params.count}` : key;

function createTurn(overrides: Partial<ConversationTurn> = {}): ConversationTurn {
  return {
    id: "ask-1",
    prompt: {
      id: "ask-1",
      conversationId: "conv-1",
      branchId: "branch-1",
      role: "user",
      createdAt: 1,
      content: { parts: [{ type: "text", text: "Summarize this note" }], citations: [] },
    },
    answers: [],
    runId: "run-1",
    active: true,
    pending: "generating",
    streamedText: "",
    ...overrides,
  };
}

function toolEvent(
  kind: "conversation_tool_started" | "conversation_tool_finished",
  sequence: number,
  toolCallId = "call-1",
): RunEvent {
  return {
    sequence,
    runId: "run-1",
    stepInstanceId: null,
    attemptId: null,
    kind,
    createdAt: 1,
    payload: {
      toolCallId,
      title: "Read note",
    },
  };
}

function mountTurn(turn: ConversationTurn, events: readonly RunEvent[] = []) {
  return mount(SpaceConversationTurn, {
    props: {
      turn,
      messages: [...(turn.prompt ? [turn.prompt] : []), ...turn.answers],
      events,
      busy: true,
    },
    global: {
      plugins: [createPinia(), i18n],
      mocks: {
        $t: translate,
      },
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

describe("SpaceConversationTurn and SpaceConversationActivity lifecycle", () => {
  it("renders the three-stroke breathing mark and shimmer for preparing and generating states", async () => {
    const wrapper = mountTurn(createTurn({ pending: "preparing", runId: undefined }));
    try {
      const shimmer = wrapper.findComponent(SpaceConversationShimmerText);
      expect(shimmer.exists()).toBe(true);
      expect(shimmer.props("indicator")).toBe(true);
      expect(shimmer.findAll(".conversation-work-mark")).toHaveLength(3);
      expect(shimmer.text()).toBe("spaces.conversation.conversations.preparing");

      await wrapper.setProps({ turn: createTurn({ pending: "generating" }) });
      expect(wrapper.findComponent(SpaceConversationShimmerText).text()).toBe(
        "spaces.conversation.activity.generating",
      );
    } finally {
      wrapper.unmount();
    }
  });

  it("transfers active status to the tool trace while tools run and returns to generating before text streams", async () => {
    const started = toolEvent("conversation_tool_started", 1);
    const finished = toolEvent("conversation_tool_finished", 2);
    const wrapper = mountTurn(createTurn({ pending: "generating" }), [started]);
    try {
      const activity = wrapper.findComponent(SpaceConversationActivity);
      expect(activity.exists()).toBe(true);
      expect(activity.find("details").attributes("open")).toBeDefined();
      expect(activity.text()).toContain("spaces.conversation.activity.running:1");
      // Never show both the active tool header and a separate "generating" line at the same time.
      expect(wrapper.text()).not.toContain("spaces.conversation.activity.generating");

      // Once all tools finish and before the first answer token arrives, the tool trace settles and
      // auto-collapses while the turn's answer slot resumes showing "generating".
      await wrapper.setProps({ events: [started, finished] });
      expect(activity.find("details").attributes("open")).toBeUndefined();
      expect(activity.text()).toContain("spaces.conversation.activity.finished:1");
      expect(wrapper.text()).toContain("spaces.conversation.activity.generating");

      // Once tokens arrive, the generating placeholder yields in place to streamed markdown and the
      // settled tool trace never reverts to "generating".
      await wrapper.setProps({
        turn: createTurn({ pending: undefined, streamedText: "Here is the summary." }),
      });
      expect(wrapper.text()).not.toContain("spaces.conversation.activity.generating");
      expect(activity.text()).toContain("spaces.conversation.activity.finished:1");
      expect(wrapper.find('[data-test="markdown"]').text()).toBe("Here is the summary.");
    } finally {
      wrapper.unmount();
    }
  });
});
