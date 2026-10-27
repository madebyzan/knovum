// @vitest-environment jsdom
// DOMPurify loses the first element of a fragment under happy-dom's parser; markdown output is
// asserted against a spec-conformant one.
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiMarkdown from "../UiMarkdown.vue";

describe("UiMarkdown citations", () => {
  it("renders only issued markers, and only openable ones as buttons", () => {
    const wrapper = mount(UiMarkdown, {
      props: {
        content: "Kept [C1] and stale [C2]; unknown [C9], code `[C1]`.",
        citations: new Map([
          [1, true],
          [2, false],
        ]),
      },
    });
    const button = wrapper.get("button.ui-citation");
    expect(button.text()).toBe("1");
    expect(wrapper.get("span.ui-citation").text()).toBe("2");
    expect(wrapper.text()).toContain("[C9]");
    expect(wrapper.get("code").text()).toBe("[C1]");
  });

  it("emits the cited index when a citation is clicked", async () => {
    const wrapper = mount(UiMarkdown, {
      props: { content: "Claim [C3].", citations: new Map([[3, true]]) },
    });
    await wrapper.get("button.ui-citation").trigger("click");
    expect(wrapper.emitted("cite")).toEqual([[3]]);
  });

  it("leaves markers as text without citations", () => {
    const wrapper = mount(UiMarkdown, { props: { content: "Claim [C3]." } });
    expect(wrapper.find(".ui-citation").exists()).toBe(false);
    expect(wrapper.text()).toBe("Claim [C3].");
  });
});

describe("UiMarkdown streaming", () => {
  it("formats text as it streams and keeps settled blocks' nodes", async () => {
    const wrapper = mount(UiMarkdown, { props: { content: "# Title\n\nFirst **para" } });
    const heading = wrapper.get("h1").element;
    expect(wrapper.get("p").html()).toBe("<p>First **para</p>");

    await wrapper.setProps({ content: "# Title\n\nFirst **paragraph**.\n\n| a | b |\n| - | - |\n| 1 | 2 |" });

    expect(wrapper.get("h1").element).toBe(heading);
    expect(wrapper.get("strong").text()).toBe("paragraph");
    expect(wrapper.get("td").text()).toBe("1");
    // Blocks are the root's own children, where the content stylesheet spaces them.
    expect([...wrapper.element.children].map((child) => child.tagName)).toEqual(["H1", "P", "TABLE"]);
  });

  it("repaints from scratch when the content is replaced rather than extended", async () => {
    const wrapper = mount(UiMarkdown, { props: { content: "One.\n\nTwo." } });

    await wrapper.setProps({ content: "Other." });

    expect(wrapper.element.innerHTML.trim()).toBe("<p>Other.</p>");
  });

  it("repaints settled blocks when their citations become known", async () => {
    const wrapper = mount(UiMarkdown, { props: { content: "Claim [C1].\n\nMore." } });
    expect(wrapper.find(".ui-citation").exists()).toBe(false);

    await wrapper.setProps({ citations: new Map([[1, true]]) });

    expect(wrapper.get("button.ui-citation").text()).toBe("1");
  });
});

describe("UiMarkdown links", () => {
  it("links only external destinations and keeps any other link's label as text", () => {
    const wrapper = mount(UiMarkdown, {
      props: {
        content:
          "Saved [PPT摘要.md](</Users/zand/agent-workspaces/PPT摘要.md>), see [docs](https://example.com) or [mail](mailto:a@example.com).",
      },
    });
    expect(wrapper.findAll("a").map((link) => link.attributes("href"))).toEqual([
      "https://example.com",
      "mailto:a@example.com",
    ]);
    expect(wrapper.text()).toContain("Saved PPT摘要.md, see docs or mail.");
  });

  it("never turns a file name into a link to a host", () => {
    const wrapper = mount(UiMarkdown, {
      props: { content: "Edit README.md and main.rs, or open https://example.com/x." },
    });
    expect(wrapper.findAll("a").map((link) => link.attributes("href"))).toEqual(["https://example.com/x"]);
  });
});
