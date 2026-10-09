import { shallowMount } from "@vue/test-utils";
import { createPinia } from "pinia";
import { describe, expect, it } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import type { SpaceCodePayload, SpaceFilePayload } from "../model/nodes/node-payload";
import SpaceCanvasCode from "../presentation/canvas/nodes/SpaceCanvasCode.vue";
import SpaceCanvasNodeBody from "../presentation/canvas/nodes/SpaceCanvasNodeBody.vue";
import SpaceCanvasNodeContent from "../presentation/canvas/nodes/SpaceCanvasNodeContent.vue";

const profiles = {
  compact: { level: "compact", previewItemLimit: 2 },
  standard: { level: "standard", previewItemLimit: 4 },
  full: { level: "full", previewItemLimit: 10 },
} as const;

function preview(level: keyof typeof profiles) {
  return { viewport: profiles[level], mode: "preview" as const };
}

describe("Konva rich node content", () => {
  it("uses the product document preview for supported document files", () => {
    const data: SpaceFilePayload = {
      kind: "file",
      title: "Research",
      assetPath: "/spaces/research.pdf",
      fileName: "research.pdf",
      previewPath: "/spaces/research-preview.png",
    };
    const wrapper = shallowMount(SpaceCanvasNodeBody, {
      props: { id: "file", data, presentationContext: preview("full"), interactive: false, frameRotation: 0 },
      global: {
        plugins: [createPinia(), i18n],
        stubs: {
          SpaceDocumentPreview: {
            name: "SpaceDocumentPreview",
            props: { fill: Boolean },
            template: '<div data-testid="document-preview" />',
          },
        },
      },
    });
    const documentPreview = wrapper.findComponent({ name: "SpaceDocumentPreview" });
    expect(documentPreview.find('[data-testid="document-preview"]').exists()).toBe(true);
    expect(documentPreview.props("fill")).toBe(true);
  });

  it("exposes credential editing only after Konva selects the full-detail node", () => {
    const wrapper = shallowMount(SpaceCanvasNodeContent, {
      props: {
        layout: {
          nodeId: "credential",
          frame: { x: 0, y: 0, width: 320, height: 180, rotation: 0 },
          sizingMode: "frame",
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
        previewPolicy: profiles.full,
        interactive: true,
        selected: true,
        persist: async () => {},
        fitContentHeight() {},
      },
      global: { plugins: [createPinia(), i18n], stubs: { SpaceCanvasCredential: true } },
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
      props: {
        id: "html",
        data: html,
        presentationContext: { viewport: profiles.full, mode: "interactive" },
        interactive: true,
        frameRotation: 0,
      },
      global: { plugins: [createPinia(), i18n] },
    });
    expect(htmlWrapper.findComponent({ name: "SpaceCanvasCode" }).exists()).toBe(true);
    const codeWrapper = shallowMount(SpaceCanvasCode, {
      props: {
        data: html,
        presentationContext: { viewport: profiles.full, mode: "interactive" },
        interactive: true,
      },
      global: { plugins: [createPinia(), i18n] },
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
        presentationContext: preview("full"),
        interactive: false,
        frameRotation: 0,
      },
      global: { plugins: [createPinia(), i18n] },
    });
    expect(processingWrapper.findComponent({ name: "SpaceCanvasProcessingState" }).exists()).toBe(true);
  });
});
