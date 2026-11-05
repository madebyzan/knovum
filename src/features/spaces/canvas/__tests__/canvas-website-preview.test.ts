import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `asset://localhost/${path}`,
}));

import { i18n } from "../../../../foundation/i18n/instance";
import SpaceCanvasWebsite from "../presentation/nodes/SpaceCanvasWebsite.vue";

const presentationContext = {
  viewport: { level: "full", previewItemLimit: 10 },
  mode: "preview",
} as const;

describe("saved website canvas node", () => {
  it("presents source, page title and description in one website node", () => {
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

  it("renders a hero preview banner with source and title when a preview image is available", () => {
    const wrapper = mount(SpaceCanvasWebsite, {
      props: {
        id: "website-4",
        data: {
          kind: "website",
          title: "Reka UI - Primitive components for Vue",
          siteName: "Reka UI",
          url: "https://reka-ui.com/",
          description: "An open-source library with unstyled, primitive components.",
          previewImagePath: "spaces/space-1/assets/preview.png",
          metadataState: "ready",
        },
        presentationContext,
      },
      global: { plugins: [i18n], stubs: { SpaceCanvasNodeStatus: true } },
    });
    const img = wrapper.find("img[src*='preview.png']");
    expect(img.exists()).toBe(true);
    expect(img.classes()).toContain("object-cover");
    expect(wrapper.text()).toContain("Reka UI");
    expect(wrapper.text()).toContain("reka-ui.com");
    expect(wrapper.text()).toContain("Reka UI - Primitive components for Vue");
    expect(wrapper.text()).toContain("An open-source library with unstyled, primitive components.");
  });

  it("renders a dedicated Twitter post for x.com / twitter.com status links", () => {
    const wrapper = mount(SpaceCanvasWebsite, {
      props: {
        id: "website-twitter",
        data: {
          kind: "website",
          title: "小弟调调 (@jaywcjlove) on X",
          siteName: "X (formerly Twitter)",
          url: "https://x.com/jaywcjlove/status/2117838024340124026",
          description: "Knovum - 本地优先的 AI 知识工作空间",
          previewImagePath: "spaces/space-1/assets/tweet-img.png",
          metadataState: "ready",
        },
        presentationContext,
      },
      global: { plugins: [i18n], stubs: { SpaceCanvasNodeStatus: true } },
    });
    expect(wrapper.text()).toContain("小弟调调");
    expect(wrapper.text()).toContain("@jaywcjlove");
    expect(wrapper.text()).toContain("Knovum - 本地优先的 AI 知识工作空间");
    expect(wrapper.text()).toContain("x.com");
    expect(wrapper.find("svg[aria-label='X']").exists()).toBe(true);
    expect(wrapper.find("img[src*='tweet-img.png']").exists()).toBe(true);
  });

  it("renders a dedicated GitHub repository view for github.com repo links", () => {
    const wrapper = mount(SpaceCanvasWebsite, {
      props: {
        id: "website-github",
        data: {
          kind: "website",
          title: "GitHub - madebyzan/knovum: The local-first AI knowledge workspace",
          siteName: "GitHub",
          url: "https://github.com/madebyzan/knovum",
          description: "The local-first AI knowledge workspace - madebyzan/knovum",
          previewImagePath: "spaces/space-1/assets/github-preview.png",
          metadataState: "ready",
        },
        presentationContext,
      },
      global: { plugins: [i18n], stubs: { SpaceCanvasNodeStatus: true } },
    });
    expect(wrapper.text()).toContain("madebyzan");
    expect(wrapper.text()).toContain("knovum");
    expect(wrapper.text()).toContain("The local-first AI knowledge workspace");
    expect(wrapper.find("svg[aria-label='GitHub']").exists()).toBe(true);
    expect(wrapper.find("img[src*='github-preview.png']").exists()).toBe(true);
  });
});
