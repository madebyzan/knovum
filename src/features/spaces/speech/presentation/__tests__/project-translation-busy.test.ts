import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";
import { createMemoryHistory, createRouter } from "vue-router";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { SpeechProjectSnapshot } from "../../../contracts/generated/SpeechProjectSnapshot";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import SpeechProjectEditor from "../SpeechProjectEditor.vue";

it("prevents synthesis while the dubbing script is being translated and releases the editor afterward", async () => {
  const project: SpeechProjectSnapshot = {
    project: {
      id: "project",
      spaceId: "space",
      sourceNodeId: "video",
      projectType: "video_dubbing",
      title: "Dubbing",
      activeRevisionId: "revision",
      activeRevision: 1,
      segmentCount: 1,
      updatedAt: 1,
    },
    revision: {
      id: "revision",
      projectId: "project",
      revision: 1,
      sourceRevision: "source",
      pronunciationLexicon: {},
      createdAt: 1,
      outputConfig: {
        format: "mp4",
        video: { assetPath: "/video.mp4", sourceRevision: `sha256:${"a".repeat(64)}`, durationMs: 1000 },
        background: null,
        backgroundSeparation: null,
        voiceGainDb: 0,
        backgroundGainDb: -12,
        translation: null,
        preserveOriginalOutsideDialogue: false,
        sourceDialogueIntervals: [],
        retainOriginalTrack: true,
      },
    },
    segments: [
      {
        id: "line",
        ordinal: 0,
        text: "Hello",
        normalizedText: "Hello",
        voiceRevisionId: "voice",
        language: "en",
        state: "ready",
      },
    ],
  };
  const sourceNode: SpaceCanvasNode = {
    id: "video",
    type: "video",
    initialPosition: { x: 0, y: 0 },
    data: { kind: "video", title: "Video", assetPath: "/video.mp4", fileName: "video.mp4" },
  };
  const wrapper = mount(SpeechProjectEditor, {
    attachTo: document.body,
    props: {
      sourceNode,
      project,
      voices: [],
      previewing: false,
      generating: false,
      submitting: false,
      saving: false,
      cancelling: false,
      previewListening: false,
      save: () => Promise.resolve(true),
    },
    global: {
      plugins: [
        i18n,
        createRouter({
          history: createMemoryHistory(),
          routes: [
            { path: "/", component: { template: "<div />" } },
            { path: "/settings", component: { template: "<div />" } },
          ],
        }),
      ],
      stubs: { VideoDubbingEditor: true, VideoDeliveryExport: true, SpeechSignalInspection: true },
    },
  });
  onTestFinished(() => wrapper.unmount());
  const delivery = () => wrapper.findComponent({ name: "VideoDeliveryExport" });
  const deliveryTrigger = wrapper
    .findAll("button")
    .find((button) => button.text() === i18n.global.t("spaces.speech.videoDubbing.delivery"))!;
  expect(deliveryTrigger.attributes("aria-expanded")).toBe("false");
  await deliveryTrigger.trigger("click");
  expect(deliveryTrigger.attributes("aria-expanded")).toBe("true");
  await wrapper.setProps({ project: { ...project, segments: [] } });
  expect(delivery().exists()).toBe(false);
  await wrapper.setProps({ project });
  const editor = wrapper.getComponent({ name: "VideoDubbingEditor" });
  const generate = wrapper.get('[data-testid="speech-generate"]');
  expect(generate.element.closest("[data-reka-scroll-area-viewport]")).toBeNull();
  expect(wrapper.find('[data-testid="speech-save"]').exists()).toBe(false);
  expect(generate.attributes("disabled")).toBeUndefined();
  await wrapper.setProps({ saving: true });
  expect(generate.attributes("disabled")).toBeDefined();
  await wrapper.setProps({ saving: false });
  editor.vm.$emit("busy", true);
  await wrapper.vm.$nextTick();
  expect(generate.attributes("disabled")).toBeDefined();
  expect(editor.props("disabled")).toBe(true);
  editor.vm.$emit("busy", false);
  await wrapper.vm.$nextTick();
  expect(generate.attributes("disabled")).toBeUndefined();
  expect(editor.props("disabled")).toBe(false);
  editor.vm.$emit("draftState", true);
  await wrapper.vm.$nextTick();
  expect(generate.attributes("disabled")).toBeDefined();
  const input = { sourceRevision: "source", outputConfig: {}, pronunciationLexicon: {}, segments: [] };
  editor.vm.$emit("draftState", true, input);
  await wrapper.vm.$nextTick();
  expect(generate.attributes("disabled")).toBeUndefined();
  await generate.trigger("click");
  expect(wrapper.emitted("generate")?.at(-1)).toEqual([input]);
  editor.vm.$emit("draftState", false, input);
  await wrapper.vm.$nextTick();
  expect(generate.attributes("disabled")).toBeUndefined();
  await generate.trigger("click");
  expect(wrapper.emitted("generate")?.at(-1)).toEqual([input]);
  editor.vm.$emit("busy", true);
  await wrapper.vm.$nextTick();
  expect(generate.attributes("disabled")).toBeDefined();
});
