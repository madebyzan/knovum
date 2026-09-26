import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import SpaceCanvasWebsite from "../presentation/canvas/nodes/SpaceCanvasWebsite.vue";

const presentationContext = {
  viewport: { level: "full", previewItemLimit: 10 },
  mode: "preview",
} as const;

describe("saved website canvas card", () => {
  it("presents source, page title and description as one knowledge card", () => {
    const wrapper = mount(SpaceCanvasWebsite, {
      props: {
        id: "website-1",
        data: {
          kind: "website",
          title: "A useful article",
          siteName: "Example Journal",
          url: "https://example.com/articles/knowledge",
          description: "A concise introduction to the article.",
          metadataState: "ready",
        },
        presentationContext,
      },
      global: { plugins: [i18n], stubs: { SpaceCanvasNodeStatus: true } },
    });
    expect(wrapper.find("iframe").exists()).toBe(false);
    expect(wrapper.text()).toContain("Example Journal");
    expect(wrapper.text()).toContain("example.com");
    expect(wrapper.text()).toContain("A useful article");
    expect(wrapper.text()).toContain("A concise introduction to the article.");
  });

  it("shows readable content when a page has no description", () => {
    const wrapper = mount(SpaceCanvasWebsite, {
      props: {
        id: "website-2",
        data: {
          kind: "website",
          title: "Documentation",
          url: "https://docs.example.com/guide",
          contentPreview: "This guide explains the main concepts.",
          metadataState: "ready",
        },
        presentationContext,
      },
      global: { plugins: [i18n], stubs: { SpaceCanvasNodeStatus: true } },
    });
    expect(wrapper.text()).toContain("This guide explains the main concepts.");
  });

  it("keeps a failed capture actionable without turning the node into a raw link", () => {
    const wrapper = mount(SpaceCanvasWebsite, {
      props: {
        id: "website-3",
        data: {
          kind: "website",
          title: "example.com",
          url: "https://example.com/",
          metadataState: "failed",
        },
        presentationContext,
      },
      global: { plugins: [i18n], stubs: { SpaceCanvasNodeStatus: true } },
    });
    expect(wrapper.text()).toContain(i18n.global.t("spaces.importExport.website.previewUnavailable"));
  });
});
