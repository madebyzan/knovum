import { enableAutoUnmount, flushPromises, mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import { afterEach, describe, expect, it, vi } from "vitest";
import { computed, nextTick, shallowRef } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import { conversationMessage } from "../../__tests__/support/conversation-transcripts";
import SpaceConversationMessage from "../SpaceConversationMessage.vue";
import { rangePassage } from "../use-answer-selection";

type Passage = { text: string; citations: readonly number[] };
const capture = vi.hoisted(() => ({
  saveAnswer: vi.fn<(markdown: string) => Promise<void>>(),
  saveSelection: vi.fn<(passage: Passage) => Promise<void>>(),
  appendSelection: vi.fn<(passage: Passage) => Promise<void>>(),
  dragSelection: vi.fn<(event: DragEvent, passage: Passage) => void>(),
}));
const appendTarget = shallowRef<{ id: string; title: string }>();
vi.mock("../../use-answer-capture", () => ({
  useAnswerCapture: () => ({ ...capture, canSave: computed(() => true), appendTarget }),
}));
vi.mock("tauri-plugin-clipboard-api", () => ({
  writeText: vi.fn<(text: string) => Promise<void>>(() => Promise.resolve()),
}));

const ANSWER = "Revenue grew 12% in Q3 [C1].\n\nCosts were flat.";

function mountAnswer() {
  const message = conversationMessage("answer", "assistant", ANSWER);
  message.content.citations = [
    {
      index: 1,
      state: "current",
      nodeId: "report",
      nodeTitle: "Q3 report",
      chunkId: "chunk",
      sourceRevision: "rev",
      locator: { sourceRevision: "rev", location: { kind: "node" } },
      quote: "Revenue grew 12%",
    },
  ];
  return mount(SpaceConversationMessage, {
    attachTo: document.body,
    props: { message, actions: true, busy: false },
    global: { plugins: [createPinia(), i18n] },
  });
}

function textNode(root: Element, containing: string) {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode())
    if (node.textContent?.includes(containing)) return node;
  throw new Error(`No text contains ${containing}`);
}

/** Selects from the start of `from` to the end of the text node holding `to`, as a drag would. */
async function select(wrapper: ReturnType<typeof mountAnswer>, from: string, to: string) {
  await flushPromises();
  const start = textNode(wrapper.element, from);
  const end = textNode(wrapper.element, to);
  const range = document.createRange();
  range.setStart(start, start.textContent!.indexOf(from));
  range.setEnd(end, end.textContent!.length);
  const selection = globalThis.getSelection()!;
  selection.removeAllRanges();
  selection.addRange(range);
  document.dispatchEvent(new Event("selectionchange"));
  await nextTick();
}

enableAutoUnmount(afterEach);
afterEach(() => {
  globalThis.getSelection()?.removeAllRanges();
  appendTarget.value = undefined;
  vi.clearAllMocks();
});

describe("capturing an answer", () => {
  it("adds the whole answer to the canvas from its actions", async () => {
    const wrapper = mountAnswer();
    await wrapper
      .get(`[aria-label="${i18n.global.t("spaces.conversation.capture.saveAnswer")}"]`)
      .trigger("click");
    expect(capture.saveAnswer).toHaveBeenCalledWith(ANSWER);
  });

  it("offers the selection bar only while text in the answer is selected", async () => {
    const wrapper = mountAnswer();
    expect(wrapper.find('[role="toolbar"]').exists()).toBe(false);
    await select(wrapper, "Revenue", "Costs");
    expect(wrapper.find('[role="toolbar"]').exists()).toBe(true);
  });

  it("saves the selected words without citation badges", async () => {
    const wrapper = mountAnswer();
    await select(wrapper, "Revenue", "Costs");
    const label = i18n.global.t("spaces.conversation.capture.saveSelection");
    await wrapper.get(`[aria-label="${label}"]`).trigger("click");
    await flushPromises();
    expect(capture.saveSelection).toHaveBeenCalledWith({
      text: "Revenue grew 12% in Q3.\nCosts were flat.",
      citations: [1],
    });
  });

  it("appends the selection to the note selected on the canvas, when there is one", async () => {
    const wrapper = mountAnswer();
    await select(wrapper, "Costs", "Costs");
    const label = i18n.global.t("spaces.conversation.capture.appendTo", { title: "Ideas" });
    expect(wrapper.find(`[aria-label="${label}"]`).exists()).toBe(false);
    appendTarget.value = { id: "note", title: "Ideas" };
    await nextTick();
    await wrapper.get(`[aria-label="${label}"]`).trigger("click");
    await flushPromises();
    expect(capture.appendSelection).toHaveBeenCalledWith({ text: "Costs were flat.", citations: [] });
  });

  it("hands a dragged selection to the canvas", async () => {
    const wrapper = mountAnswer();
    await select(wrapper, "Costs", "Costs");
    await wrapper.get(".ui-markdown").trigger("dragstart");
    expect(capture.dragSelection).toHaveBeenCalledWith(expect.any(Event), {
      text: "Costs were flat.",
      citations: [],
    });
  });

  it("quotes the selection for a follow-up question", async () => {
    const wrapper = mountAnswer();
    await select(wrapper, "Costs", "Costs");
    await wrapper
      .get(`[aria-label="${i18n.global.t("spaces.conversation.capture.followUp")}"]`)
      .trigger("click");
    expect(wrapper.emitted("quote")?.[0]).toEqual(["Costs were flat."]);
  });

  it("reads each rendered block as its own line", () => {
    const host = document.createElement("div");
    host.innerHTML =
      '<h2>Plan</h2><p>Grew <button class="ui-citation" data-citation-index="2">2</button>, then fell.</p><ul><li>One</li><li>Two</li></ul>';
    const range = document.createRange();
    range.selectNodeContents(host);
    expect(rangePassage(range, host)).toEqual({
      text: "Plan\n\nGrew, then fell.\n\nOne\n\nTwo",
      citations: [2],
    });
  });

  it("takes the citations of the blocks a selection touches, not those of other blocks", () => {
    const host = document.createElement("div");
    host.innerHTML =
      '<p>Revenue grew <button class="ui-citation" data-citation-index="1">1</button></p>' +
      '<p>Costs fell <button class="ui-citation" data-citation-index="2">2</button> <span class="ui-citation" data-citation-detached="">3</span></p>';
    document.body.append(host);
    const claim = host.querySelector("p")!.firstChild!;
    const range = document.createRange();
    range.setStart(claim, 0);
    range.setEnd(claim, "Revenue".length);
    expect(rangePassage(range, host)).toEqual({ text: "Revenue", citations: [1] });
    range.setEnd(host.lastElementChild!.firstChild!, 5);
    expect(rangePassage(range, host)?.citations).toEqual([1, 2]);
    host.remove();
  });
});
