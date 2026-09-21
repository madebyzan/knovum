import type { Editor } from "@tiptap/core";
import { EditorContent } from "@tiptap/vue-3";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, nextTick } from "vue";

import { i18n } from "../../../foundation/i18n/i18n";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceCanvasTextPayload } from "../model/nodes/capabilities";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import SpaceCanvasMedia from "../presentation/canvas/nodes/SpaceCanvasMedia.vue";
import SpaceCanvasNodeStatus from "../presentation/canvas/nodes/SpaceCanvasNodeStatus.vue";
import SpaceCanvasNote from "../presentation/canvas/nodes/SpaceCanvasNote.vue";
import { useSpaceStore } from "../session/space-store";

describe("Konva rich node interactions", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("persists direct note editing and refuses editing for locked notes", async () => {
    const persist = vi.fn<(id: string, data: SpaceCanvasTextPayload) => Promise<void>>(async () => {});
    const wrapper = mount(SpaceCanvasNote, {
      props: {
        id: "note",
        data: createSpaceNotePayload("First", "First\nBody"),
        interactive: false,
        verticalClass: "justify-start",
        fontSize: 18,
        textAlign: "left",
        persist,
      },
      global: { plugins: [i18n] },
    });

    await wrapper.setProps({ interactive: true });
    await nextTick();
    const editor = wrapper.findComponent(EditorContent).props("editor") as Editor;
    editor.commands.setContent({
      type: "doc",
      content: [
        { type: "paragraph", content: [{ type: "text", text: "Revised" }] },
        {
          type: "paragraph",
          content: [{ type: "text", marks: [{ type: "bold" }], text: "Content" }],
        },
      ],
    });
    await wrapper.setProps({ interactive: false });
    await flushPromises();
    expect(persist).toHaveBeenCalledWith(
      "note",
      expect.objectContaining({
        title: "Revised",
        plainText: "Revised\nContent",
        tiptapContent: expect.objectContaining({
          content: expect.arrayContaining([
            expect.objectContaining({ type: "paragraph" }),
            expect.objectContaining({
              content: [expect.objectContaining({ marks: [{ type: "bold" }] })],
            }),
          ]),
        }),
      }),
    );

    await wrapper.setProps({ data: { ...createSpaceNotePayload("Locked", "Body"), locked: true } });
    expect(wrapper.find('[tabindex="0"]').exists()).toBe(false);
    expect(wrapper.find(".ProseMirror[contenteditable='true']").exists()).toBe(false);
  });

  it("consumes media requests and synchronizes playback state", async () => {
    const togglePlayback = vi.fn<() => Promise<void>>(async () => {});
    const seekTo = vi.fn<(positionMs: number) => Promise<void>>(async () => {});
    const player = defineComponent({
      emits: ["playback", "unload", "playbackError"],
      setup(_, { expose }) {
        expose({ togglePlayback, seekTo });
        return () => null;
      },
    });
    const data: SpaceVideoPayload = {
      kind: "video",
      title: "Interview",
      assetPath: "/spaces/interview.mp4",
      fileName: "interview.mp4",
    };
    const wrapper = mount(SpaceCanvasMedia, {
      props: { id: "video", data },
      global: { plugins: [i18n], stubs: { SpaceMediaPlayer: player } },
    });
    const store = useSpaceStore();

    store.requestMediaSeek("video", 1_250);
    await nextTick();
    await flushPromises();
    expect(seekTo).toHaveBeenCalledWith(1_250);
    expect(store.mediaSeekRequest).toBeUndefined();

    store.requestMediaToggle("video");
    await nextTick();
    await flushPromises();
    expect(togglePlayback).toHaveBeenCalledOnce();
    expect(store.mediaToggleRequest).toBeUndefined();

    const playback = { currentMs: 1_250, durationMs: 5_000, playing: true };
    wrapper.findComponent(player).vm.$emit("playback", playback);
    await nextTick();
    expect(store.mediaPlayback).toEqual({ id: "video", state: playback });
  });

  it("surfaces background media analysis on the canvas node", () => {
    const store = useSpaceStore();
    store.mediaAnalysisIds.push("video");
    const wrapper = mount(SpaceCanvasNodeStatus, {
      props: {
        id: "video",
        data: {
          kind: "video",
          title: "Interview",
          assetPath: "/spaces/interview.mp4",
          fileName: "interview.mp4",
        },
      },
      global: { plugins: [i18n] },
    });

    expect(wrapper.get('[role="status"]').attributes("aria-label")).toBe(
      i18n.global.t("spaces.media.analysisInProgress"),
    );
  });
});
