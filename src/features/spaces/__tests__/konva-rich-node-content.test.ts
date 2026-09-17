import { shallowMount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import type {
  SpaceAudioPayload,
  SpaceCodePayload,
  SpaceFilePayload,
  SpaceVideoPayload,
} from "../model/nodes/node-payload";
import SpaceCanvasCode from "../presentation/canvas/nodes/SpaceCanvasCode.vue";
import SpaceCanvasNodeBody from "../presentation/canvas/nodes/SpaceCanvasNodeBody.vue";
import SpaceCanvasNodeContent from "../presentation/canvas/nodes/SpaceCanvasNodeContent.vue";

describe("Konva rich node content", () => {
  it("presents audio transport instead of repeating its title as body text", () => {
    const data: SpaceAudioPayload = {
      kind: "audio",
      title: "Daily reminder",
      assetPath: "/spaces/reminder.mp3",
      fileName: "reminder.mp3",
    };
    const wrapper = shallowMount(SpaceCanvasNodeBody, {
      props: { id: "audio", data, detailLevel: "summary", interactive: false },
      global: {
        plugins: [i18n],
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
      props: { id: "video", data, detailLevel: "full", interactive: true },
      global: {
        plugins: [i18n],
        stubs: { SpaceCanvasMedia: { template: '<div data-testid="media-player" />' } },
      },
    });
    expect(wrapper.find('[data-testid="media-player"]').exists()).toBe(true);
  });

  it("keeps selection passive and enters full media detail only after explicit activation", async () => {
    const data: SpaceVideoPayload = {
      kind: "video",
      title: "Interview",
      assetPath: "/spaces/interview.mp4",
      fileName: "interview.mp4",
    };
    const wrapper = shallowMount(SpaceCanvasNodeContent, {
      props: {
        layout: {
          nodeId: "video",
          frame: { x: 0, y: 0, width: 420, height: 368, rotation: 0 },
          sizeAuthority: "frame",
          zIndex: 0,
        },
        node: { id: "video", type: "video", data, initialPosition: { x: 0, y: 0 } },
        detailLevel: "summary",
        interactive: false,
        selected: true,
        persist: async () => {},
        fitContentHeight() {},
      },
    });
    const body = wrapper.findComponent({ name: "SpaceCanvasNodeBody" });
    expect(body.props("detailLevel")).toBe("summary");
    expect(body.props("interactive")).toBe(false);
    await wrapper.setProps({ interactive: true });
    expect(body.props("detailLevel")).toBe("full");
    expect(body.props("interactive")).toBe(true);
  });

  it("uses the product document preview for supported document files", () => {
    const data: SpaceFilePayload = {
      kind: "file",
      title: "Research",
      assetPath: "/spaces/research.pdf",
      fileName: "research.pdf",
      previewPath: "/spaces/research-preview.png",
    };
    const wrapper = shallowMount(SpaceCanvasNodeBody, {
      props: { id: "file", data, detailLevel: "full", interactive: false },
      global: {
        plugins: [i18n],
        stubs: {
          SpaceDocumentPreview: {
            name: "SpaceDocumentPreview",
            props: { fill: Boolean },
            template: '<div data-testid="document-preview" />',
          },
        },
      },
    });
    const preview = wrapper.findComponent({ name: "SpaceDocumentPreview" });
    expect(preview.find('[data-testid="document-preview"]').exists()).toBe(true);
    expect(preview.props("fill")).toBe(true);
  });

  it("exposes credential editing only after Konva selects the full-detail node", () => {
    const wrapper = shallowMount(SpaceCanvasNodeContent, {
      props: {
        layout: {
          nodeId: "credential",
          frame: { x: 0, y: 0, width: 320, height: 180, rotation: 0 },
          sizeAuthority: "frame",
          zIndex: 0,
        },
        node: {
          id: "credential",
          type: "credential",
          data: {
            kind: "credential",
            title: "Publishing API",
            credentialKind: "api-key",
            credentialConfigured: true,
          },
          initialPosition: { x: 0, y: 0 },
        },
        detailLevel: "full",
        interactive: true,
        selected: true,
        persist: async () => {},
        fitContentHeight() {},
      },
      global: { stubs: { SpaceCanvasCredential: true } },
    });
    expect(wrapper.findComponent({ name: "SpaceCanvasCredential" }).props("interactive")).toBe(true);
  });

  it("keeps HTML preview and processing state as first-class node presentations", async () => {
    const html: SpaceCodePayload = {
      kind: "code",
      title: "Page",
      assetPath: "/spaces/page.html",
      fileName: "page.html",
      previewLanguage: "html",
      previewPath: "/spaces/page-preview.html",
      previewText: "<h1>Page</h1>",
    };
    const htmlWrapper = shallowMount(SpaceCanvasNodeBody, {
      props: { id: "html", data: html, detailLevel: "full", interactive: true },
      global: { plugins: [i18n] },
    });
    expect(htmlWrapper.findComponent({ name: "SpaceCanvasCode" }).exists()).toBe(true);
    const codeWrapper = shallowMount(SpaceCanvasCode, {
      props: { data: html, detailLevel: "full", interactive: true },
      global: { plugins: [i18n] },
    });
    expect(codeWrapper.findComponent({ name: "SpaceHtmlPreview" }).exists()).toBe(false);
    await codeWrapper.findComponent({ name: "UiIconButton" }).trigger("click");
    expect(codeWrapper.findComponent({ name: "SpaceHtmlPreview" }).exists()).toBe(true);

    const processingWrapper = shallowMount(SpaceCanvasNodeBody, {
      props: {
        id: "file",
        data: {
          kind: "file",
          title: "Import",
          assetPath: "/spaces/import.pdf",
          fileName: "import.pdf",
          processingState: "processing",
        },
        detailLevel: "full",
        interactive: false,
      },
      global: { plugins: [i18n] },
    });
    expect(processingWrapper.findComponent({ name: "SpaceCanvasProcessingState" }).exists()).toBe(true);
  });
});
