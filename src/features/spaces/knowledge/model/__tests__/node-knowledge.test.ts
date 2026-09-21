import { describe, expect, it } from "vitest";

import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import { prepareSpaceKnowledgePayload } from "../node-knowledge";
import { knowledgeRevision } from "../revision";

describe("Knowledge revision preparation", () => {
  it("ignores local Tiptap asset paths and changes when semantic text changes", () => {
    const left = {
      kind: "note",
      title: "Note",
      plainText: "hello",
      tiptapContent: {
        type: "doc",
        content: [{ type: "image", attrs: { src: "asset://a", assetPath: "/a", alt: "x" } }],
      },
    } as SpaceNodePayload;
    const right = {
      ...left,
      tiptapContent: {
        type: "doc",
        content: [{ type: "image", attrs: { src: "asset://b", assetPath: "/b", alt: "x" } }],
      },
    } as SpaceNodePayload;
    expect(knowledgeRevision(left)).toBe(knowledgeRevision(right));
    const prepared = prepareSpaceKnowledgePayload(left);
    expect(prepared.knowledgeRevision).toMatch(/^node1:[0-9a-f]{16}$/u);
    expect(knowledgeRevision({ ...left, plainText: "changed" } as SpaceNodePayload)).not.toBe(
      prepared.knowledgeRevision,
    );
  });

  it("keeps remote image URLs in the semantic revision", () => {
    const left = {
      kind: "note",
      title: "Note",
      plainText: "",
      tiptapContent: {
        type: "doc",
        content: [{ type: "image", attrs: { src: "https://a.example/image.png", alt: "x" } }],
      },
    } as SpaceNodePayload;
    const right = {
      ...left,
      tiptapContent: {
        type: "doc",
        content: [{ type: "image", attrs: { src: "https://b.example/image.png", alt: "x" } }],
      },
    } as SpaceNodePayload;
    expect(knowledgeRevision(left)).not.toBe(knowledgeRevision(right));
  });

  it("uses formatter-independent IEEE-754 canonical numbers", () => {
    const data = {
      kind: "object",
      title: "N",
      tags: [],
      objectTypeId: "system:test",
      summary: "",
      status: "draft",
      properties: { n: 1e20 },
    } as unknown as SpaceNodePayload;
    expect(knowledgeRevision(data)).toBe("node1:ecbb9a3affd2948d");
    const negativeZero = { ...data, properties: { n: -0 } } as unknown as SpaceNodePayload;
    const positiveZero = { ...data, properties: { n: 0 } } as unknown as SpaceNodePayload;
    expect(knowledgeRevision(negativeZero)).toBe(knowledgeRevision(positiveZero));
  });

  it("matches Rust UTF-8 canonical ordering for non-BMP property keys", () => {
    const data = {
      kind: "object",
      title: "O",
      tags: [],
      objectTypeId: "system:test",
      summary: "",
      status: "draft",
      properties: { 𐀀: "a", "": "b" },
    } as unknown as SpaceNodePayload;
    expect(knowledgeRevision(data)).toBe("node1:526d1abe2260909e");
  });
});
