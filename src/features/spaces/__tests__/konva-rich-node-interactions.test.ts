import type { Editor } from "@tiptap/core";
import { EditorContent } from "@tiptap/vue-3";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, nextTick, shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import SpaceCanvasMedia from "../canvas/presentation/nodes/SpaceCanvasMedia.vue";
import SpaceCanvasNodeStatus from "../canvas/presentation/nodes/SpaceCanvasNodeStatus.vue";
import SpaceCanvasNote from "../canvas/presentation/nodes/SpaceCanvasNote.vue";
import type { EnrichmentJobProjection } from "../contracts/generated/EnrichmentJobProjection";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import { enrichmentProjectionKey } from "../enrichment/enrichment-projection-key";
import type { SpaceCanvasTextPayload } from "../model/nodes/capabilities";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { useSpaceStore } from "../session/space-store";
import { openStoreSpaceTab } from "./support/session-context";

describe("Konva rich node interactions", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("persists direct note editing", async () => {
    const persist = vi.fn<(nodeId: string, data: SpaceCanvasTextPayload) => Promise<void>>(async () => {});
    const wrapper = mount(SpaceCanvasNote, {
      props: {
        id: "note",
        data: createSpaceNotePayload("First", "First\nBody"),
        presentationContext: {
          viewport: { level: "full", previewItemLimit: 10 },
          mode: "preview",
        },
        fontSize: 16,
        interactive: false,
        persist,
        textAlign: "left",
        verticalClass: "justify-start",
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
    document.body.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true }));
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
      global: { plugins: [i18n], provide: { [enrichmentProjectionKey]: shallowRef([]) } },
    });

    expect(wrapper.get('[role="status"]').attributes("aria-label")).toBe(
      i18n.global.t("spaces.media.analysis.inProgress"),
    );
  });

  it("identifies a visual analysis job on its own canvas node", () => {
    const store = useSpaceStore();
    openStoreSpaceTab(store, "space-1");
    const record = {
      job: {
        id: "job-1",
        spaceId: "space-1",
        nodeId: "image-1",
        jobType: "analyze_visual",
        status: "failed",
        sourceRevision: "revision-1",
        attempts: 1,
        executionTarget: null,
        error: "Model unavailable",
        createdAt: 1,
        updatedAt: 2,
      },
      spaceTitle: "Research",
      nodeTitle: "Cover",
      nodeKind: "image",
    } satisfies EnrichmentJobProjection;
    const wrapper = mount(SpaceCanvasNodeStatus, {
      props: {
        id: "image-1",
        data: { kind: "image", title: "Cover", assetPath: "/spaces/cover.png", fileName: "cover.png" },
      },
      global: { plugins: [i18n], provide: { [enrichmentProjectionKey]: shallowRef([record]) } },
    });

    expect(wrapper.get('[role="status"]').attributes("aria-label")).toContain(
      i18n.global.t("spaces.processing.enrichmentJobType.analyze_visual"),
    );
    expect(wrapper.get('[role="status"]').attributes("aria-label")).toContain(
      i18n.global.t("spaces.processing.jobStatus.failed"),
    );
  });
});
