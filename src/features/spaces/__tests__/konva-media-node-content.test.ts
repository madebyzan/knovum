import { shallowMount } from "@vue/test-utils";
import { createPinia } from "pinia";
import { describe, expect, it } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import type { SpaceAudioPayload, SpaceVideoPayload } from "../model/nodes/node-payload";
import SpaceCanvasNodeBody from "../presentation/canvas/nodes/SpaceCanvasNodeBody.vue";
import SpaceCanvasNodeContent from "../presentation/canvas/nodes/SpaceCanvasNodeContent.vue";

const profiles = {
  summary: { level: "summary", previewItemLimit: 4 },
  full: { level: "full", previewItemLimit: 10 },
} as const;

describe("Konva media node content", () => {
  it("presents audio transport instead of repeating its title as body text", () => {
    const data: SpaceAudioPayload = {
      kind: "audio",
      title: "Daily reminder",
      assetPath: "/spaces/reminder.mp3",
      fileName: "reminder.mp3",
    };
    const wrapper = shallowMount(SpaceCanvasNodeBody, {
      props: {
        id: "audio",
        data,
        presentationContext: { viewport: profiles.summary, mode: "preview" },
        interactive: false,
      },
      global: {
        plugins: [createPinia(), i18n],
        stubs: { SpaceCanvasMedia: { template: '<div data-testid="audio-transport" />' } },
      },
    });

    expect(wrapper.find('[data-testid="audio-transport"]').exists()).toBe(true);
    expect(wrapper.text()).not.toContain(data.title);
  });

  it("uses the product media player for an interactive full-detail video", () => {
    const data: SpaceVideoPayload = {
      kind: "video",
      title: "Interview",
      assetPath: "/spaces/interview.mp4",
      fileName: "interview.mp4",
    };
    const wrapper = shallowMount(SpaceCanvasNodeBody, {
      props: {
        id: "video",
        data,
        presentationContext: { viewport: profiles.full, mode: "interactive" },
        interactive: true,
      },
      global: {
        plugins: [createPinia(), i18n],
        stubs: { SpaceCanvasMedia: { template: '<div data-testid="media-player" />' } },
      },
    });
    expect(wrapper.find('[data-testid="media-player"]').exists()).toBe(true);
  });

  it("keeps viewport density stable while explicit activation reveals interactive content", async () => {
    const data: SpaceVideoPayload = {
      kind: "video",
      title: "Interview",
      assetPath: "/spaces/interview.mp4",
      fileName: "interview.mp4",
    };
    const wrapper = shallowMount(SpaceCanvasNodeContent, {
      global: { plugins: [createPinia(), i18n] },
      props: {
        layout: {
          nodeId: "video",
          frame: { x: 0, y: 0, width: 420, height: 368, rotation: 0 },
          sizeAuthority: "frame",
          zIndex: 0,
        },
        node: { id: "video", type: "video", data, initialPosition: { x: 0, y: 0 } },
        previewPolicy: profiles.summary,
        interactive: false,
        selected: true,
        persist: async () => {},
        fitContentHeight() {},
      },
    });
    const body = wrapper.findComponent({ name: "SpaceCanvasNodeBody" });
    expect(body.props("presentationContext")).toEqual({
      viewport: profiles.summary,
      mode: "preview",
    });
    await wrapper.setProps({ interactive: true });
    expect(body.props("presentationContext")).toEqual({
      viewport: profiles.summary,
      mode: "interactive",
    });
  });
});
