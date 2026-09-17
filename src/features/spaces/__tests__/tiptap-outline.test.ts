import { describe, expect, it } from "vitest";

import { activeHeadingFromScroll, spaceTiptapHeadings } from "../editor/tiptap-outline";

function rect(top: number): DOMRect {
  return { x: 0, y: top, width: 0, height: 0, top, right: 0, bottom: top, left: 0, toJSON: () => ({}) };
}

describe("spaces Tiptap outline", () => {
  it("collects H1-H6 headings with stable document positions", () => {
    const nodes = [
      { type: { name: "paragraph" }, attrs: {}, textContent: "Intro copy", position: 0 },
      { type: { name: "heading" }, attrs: { level: 1 }, textContent: "Introduction", position: 2 },
      { type: { name: "heading" }, attrs: { level: 2 }, textContent: "Storage", position: 20 },
      { type: { name: "heading" }, attrs: { level: 4 }, textContent: "Details", position: 42 },
      { type: { name: "heading" }, attrs: { level: 6 }, textContent: "Fine print", position: 58 },
    ];
    const editor = {
      state: {
        doc: {
          descendants(visitor: (node: (typeof nodes)[number], position: number) => void) {
            for (const node of nodes) visitor(node, node.position);
          },
        },
      },
      view: { nodeDOM: () => null },
    };

    expect(spaceTiptapHeadings(editor)).toEqual([
      { id: "heading-2", level: 1, text: "Introduction", position: 2 },
      { id: "heading-20", level: 2, text: "Storage", position: 20 },
      { id: "heading-42", level: 4, text: "Details", position: 42 },
      { id: "heading-58", level: 6, text: "Fine print", position: 58 },
    ]);
  });

  it("tracks the last heading above the scroll activation line", () => {
    const first = document.createElement("h1");
    const second = document.createElement("h2");
    first.getBoundingClientRect = () => rect(80);
    second.getBoundingClientRect = () => rect(180);
    const root = document.createElement("div");
    root.getBoundingClientRect = () => rect(0);
    Object.defineProperties(root, {
      clientHeight: { value: 600 },
      scrollHeight: { value: 1200 },
      scrollTop: { value: 200, writable: true },
    });
    const headings = [
      { id: "heading-2", level: 1 as const, text: "Introduction", position: 2 },
      { id: "heading-20", level: 2 as const, text: "Storage", position: 20 },
    ];
    const editor = {
      state: { doc: { descendants: () => undefined } },
      view: { nodeDOM: (position: number) => (position === 2 ? first : second) },
    };

    expect(activeHeadingFromScroll(editor, headings, root)).toBe("heading-2");
  });

  it("keeps the first heading active before it crosses the activation line", () => {
    const first = document.createElement("h1");
    first.getBoundingClientRect = () => rect(300);
    const root = document.createElement("div");
    root.getBoundingClientRect = () => rect(0);
    Object.defineProperties(root, {
      clientHeight: { value: 600 },
      scrollHeight: { value: 1200 },
      scrollTop: { value: 0, writable: true },
    });
    const headings = [{ id: "heading-2", level: 1 as const, text: "Introduction", position: 2 }];
    const editor = {
      state: { doc: { descendants: () => undefined } },
      view: { nodeDOM: () => first },
    };

    expect(activeHeadingFromScroll(editor, headings, root)).toBe("heading-2");
  });
});
