import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, onTestFinished } from "vitest";

import { i18n } from "../../../../../foundation/i18n/i18n";
import type { SpaceTiptapHeading } from "../../../editor/tiptap-outline";
import SpaceDocumentOutline from "../SpaceDocumentOutline.vue";

const headings: SpaceTiptapHeading[] = Array.from({ length: 24 }, (_, index) => ({
  id: `heading-${index}`,
  level: index % 3 === 0 ? 1 : 3,
  text: `Section ${index + 1}`,
  position: index * 10,
}));

function render(items = headings) {
  const wrapper = mount(SpaceDocumentOutline, {
    attachTo: document.body,
    props: { headings: items, activeId: "heading-3" },
    global: { plugins: [i18n] },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

describe("Document outline minibar", () => {
  it("keeps the full directory collapsed and limits orientation to major headings", () => {
    const wrapper = render();
    expect(wrapper.find("nav").exists()).toBe(false);
    expect(wrapper.get("button").findAll("span")).toHaveLength(8);
  });

  it("expands on hover and navigates every heading within the editor", async () => {
    const wrapper = render();
    await wrapper.get('[data-testid="document-outline"]').trigger("mouseenter");
    await flushPromises();
    const entries = wrapper.get("nav").findAll("button");
    expect(entries).toHaveLength(24);
    expect(entries[3]!.attributes("data-active")).toBe("true");
    await entries[20]!.trigger("click");
    expect(wrapper.emitted("select")).toEqual([[200]]);
  });

  it("caps marks for documents without major headings and collapses after hover", async () => {
    const wrapper = render(headings.map(({ id, text, position }) => ({ id, text, position, level: 3 })));
    expect(wrapper.get("button").findAll("span")).toHaveLength(16);
    await wrapper.get('[data-testid="document-outline"]').trigger("mouseenter");
    await flushPromises();
    expect(wrapper.find("nav").exists()).toBe(true);
    await wrapper.get('[data-testid="document-outline"]').trigger("mouseleave");
    await flushPromises();
    expect(wrapper.find("nav").exists()).toBe(false);
  });

  it("allows explicit activation without hovering", async () => {
    const wrapper = render();
    await wrapper.get("button").trigger("click");
    await flushPromises();
    expect(wrapper.get("nav").findAll("button")).toHaveLength(24);
  });

  it("omits the minibar when the document has no headings", () => {
    expect(render([]).find("button").exists()).toBe(false);
  });
});
