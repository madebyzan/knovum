import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import { i18n } from "../../../../foundation/i18n/instance";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";

const space = vi.hoisted(() => ({ nodes: [] as SpaceCanvasNode[], requestReveal: () => undefined }));
vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => `asset://localhost${path}` }));
vi.mock("../../session/space-store", () => ({ useSpaceStore: () => space }));

import SpaceConversationOutputs from "../presentation/SpaceConversationOutputs.vue";
import SpaceConversationSentAttachment from "../presentation/SpaceConversationSentAttachment.vue";

const deck = {
  id: "deck",
  data: {
    kind: "file",
    title: "Git Branch Specification.pptx",
    fileName: "Git Branch Specification.pptx",
    assetPath: "/assets/deck.pptx",
    previewPath: "/previews/deck.png",
    sourceRevision: "r1",
  },
} as unknown as SpaceCanvasNode;

function pictures(wrapper: ReturnType<typeof mount>) {
  return wrapper.findAll("img").map((image) => image.attributes("src"));
}

describe("conversation attachment pictures", () => {
  it("shows a sent canvas node as it looks on the canvas, whatever its kind", () => {
    space.nodes = [deck];
    const wrapper = mount(SpaceConversationSentAttachment, {
      props: {
        attachment: { source: "node", nodeId: "deck", title: "Git Branch Specification.pptx", kind: "file" },
      },
      global: { plugins: [i18n] },
    });
    expect(pictures(wrapper)).toEqual(["asset://localhost/previews/deck.png?v=r1"]);
  });

  it("shows a node no longer on the canvas by its kind", () => {
    space.nodes = [];
    const wrapper = mount(SpaceConversationSentAttachment, {
      props: { attachment: { source: "node", nodeId: "deck", title: "deck.pptx", kind: "file" } },
      global: { plugins: [i18n] },
    });
    expect(pictures(wrapper)).toEqual([]);
  });

  it("keeps the picture an uploaded file was sent with: nothing else holds that file", () => {
    space.nodes = [];
    const wrapper = mount(SpaceConversationSentAttachment, {
      props: {
        attachment: {
          source: "file",
          title: "dog.png",
          kind: "image",
          thumbnail: "data:image/png;base64,AA",
        },
      },
      global: { plugins: [i18n] },
    });
    expect(pictures(wrapper)).toEqual(["data:image/png;base64,AA"]);
  });

  it("draws what a reply added the same way", () => {
    space.nodes = [deck];
    const wrapper = mount(SpaceConversationOutputs, {
      props: { outputs: [{ nodeId: "deck", title: "Git Branch Specification.pptx", kind: "file" }] },
      global: { plugins: [i18n] },
    });
    expect(pictures(wrapper)).toEqual(["asset://localhost/previews/deck.png?v=r1"]);
  });
});
