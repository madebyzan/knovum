import { shallowMount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { nextTick } from "vue";

import { i18n } from "../../../../foundation/i18n/instance";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceDocumentCoverage from "../presentation/documents/SpaceDocumentCoverage.vue";

const resourceId = "a".repeat(64);
const node: SpaceCanvasNode = {
  id: "document-1",
  type: "file",
  initialPosition: { x: 0, y: 0 },
  data: {
    kind: "file",
    title: "Report",
    assetPath: "/assets/report.pdf",
    fileName: "report.pdf",
    sourceRevision: "revision-1",
    documentCoverage: {
      schemaVersion: 1,
      parser: "pdf-inspector",
      sourceRevision: "revision-1",
      pageCount: 1,
      pagesRequiringRecognition: [],
      pagesWithExtractedText: [1],
      pagesWithoutExtractableContent: [],
      resources: [
        {
          id: resourceId,
          path: "/assets/document-resource.png",
          mediaType: "image/png",
          width: 800,
          height: 600,
          byteSize: 2048,
        },
      ],
      occurrences: [
        {
          id: "pdf:1:0",
          location: { kind: "pdf", page: 1 },
          context: "Figure context",
          altText: null,
          resourceId,
          pdfInput: {
            kind: "image",
            page: 1,
            rect: { x: 0, y: 0, width: 200, height: 100 },
          },
          state: "pending",
          error: null,
          analysis: null,
        },
      ],
      warnings: ["pdf_visual_coverage_limited"],
    },
  },
};

describe("document coverage details", () => {
  it("opens the shared resource viewer from the projected resource grid", async () => {
    const wrapper = shallowMount(SpaceDocumentCoverage, {
      props: { node },
      global: { plugins: [i18n] },
    });
    const grid = wrapper.findComponent({ name: "SpaceDocumentResourceGrid" });
    const resource = grid.props("resources")[0];

    grid.vm.$emit("select", resource);
    await nextTick();

    const viewer = wrapper.findComponent({ name: "SpaceDocumentResourceViewer" });
    expect(viewer.exists()).toBe(true);
    expect(viewer.props("item")).toEqual(resource);
    expect(viewer.props("sourceRevision")).toBe("revision-1");

    await wrapper.setProps({ node: { ...node, id: "document-2" } });
    expect(wrapper.findComponent({ name: "SpaceDocumentResourceViewer" }).exists()).toBe(false);
  });
});
