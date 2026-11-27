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

  it("renders non-interactive preview badges without mutating text or emitting cite", async () => {
    const wrapper = mount(UiMarkdown, {
      props: { content: "Claim [C5][C43], code `[C1]`.", citations: "preview" },
    });
    const badges = wrapper.findAll("span.ui-citation[data-citation-preview]");
    expect(badges.map((badge) => badge.text())).toEqual(["5", "43"]);
    expect(wrapper.get("code").text()).toBe("[C1]");
    await badges[0]!.trigger("click");
    expect(wrapper.emitted("cite")).toBeUndefined();
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

  it("keeps a local link for a surface that follows it, and reports the press instead of navigating", async () => {
    const wrapper = mount(UiMarkdown, {
      props: {
        content: "Saved [PPT摘要.md](</Users/zand/PPT摘要.md>) and [docs](https://example.com).",
        localLinks: true,
      },
      attachTo: document.body,
    });
    const [local, external] = wrapper.findAll("a");
    expect(local?.attributes("href")).toBe("/Users/zand/PPT%E6%91%98%E8%A6%81.md");
    const press = new MouseEvent("click", { bubbles: true, cancelable: true });
    local?.element.dispatchEvent(press);
    expect(press.defaultPrevented).toBe(true);
    await external?.trigger("click");
    expect(wrapper.emitted("link")).toEqual([["/Users/zand/PPT%E6%91%98%E8%A6%81.md"]]);
    wrapper.unmount();
  });

  it("never turns a file name into a link to a host", () => {
    const wrapper = mount(UiMarkdown, {
      props: { content: "Edit README.md and main.rs, or open https://example.com/x." },
    });
    expect(wrapper.findAll("a").map((link) => link.attributes("href"))).toEqual(["https://example.com/x"]);
  });
});

describe("UiMarkdown rich blocks (KaTeX & Callout)", () => {
  it("renders inline and block math elements while keeping currency intact", () => {
    const wrapper = mount(UiMarkdown, {
      props: {
        content: "Price is $100$ and energy is $E = mc^2$.\n\n$$\n\\int_0^1 x^2 \\, dx\n$$",
      },
    });
    const inline = wrapper.find('[data-type="inline-math"]');
    const block = wrapper.find('[data-type="block-math"]');
    expect(inline.attributes("data-latex")).toBe("E = mc^2");
    expect(block.attributes("data-latex")).toBe("\\int_0^1 x^2 \\, dx");
    expect(wrapper.text()).toContain("$100$");
  });

  it("transforms GitHub-style alert blockquotes into semantic callout containers", () => {
    const wrapper = mount(UiMarkdown, {
      props: { content: "> [!TIP]\n> Prefer live previews everywhere." },
    });
    const callout = wrapper.get('[data-type="callout"]');
    expect(callout.attributes("data-callout-kind")).toBe("tip");
    expect(callout.text()).toContain("Prefer live previews everywhere.");
  });

  it("renders GitHub task lists with semantic checkboxes and checked state", () => {
    const wrapper = mount(UiMarkdown, {
      props: { content: "- [ ] Pending item\n- [x] Finished item" },
    });
    const items = wrapper.findAll('ul[data-type="taskList"] > li');
    expect(items).toHaveLength(2);
    expect(items[0]?.attributes("data-checked")).toBe("false");
    expect(items[1]?.attributes("data-checked")).toBe("true");
    expect(wrapper.findAll('ul[data-type="taskList"] input[type="checkbox"]')).toHaveLength(2);
    expect(wrapper.text()).not.toContain("[ ]");
    expect(wrapper.text()).not.toContain("[x]");
  });

  it("preserves Tauri asset:// image URLs in Markdown preview and reading mode", () => {
    const wrapper = mount(UiMarkdown, {
      props: {
        content: "![小狐狸](asset://localhost/%2Ftmp%2Ffox.png)",
      },
    });
    const img = wrapper.get("img");
    expect(img.attributes("src")).toBe("asset://localhost/%2Ftmp%2Ffox.png");
    expect(img.attributes("alt")).toBe("小狐狸");
  });

  it("renders native video players for Markdown video URLs and streaming placeholders for pending generation directives", () => {
    const wrapper = mount(UiMarkdown, {
      props: {
        content: [
          "![星夜动画](asset://localhost/%2Ftmp%2Fstarlight.mp4)",
          "![小狐狸提灯](knovum-gen://image?prompt=fox+with+lantern&aspect=16:9)",
          "![灯笼微光](knovum-gen://video?prompt=glowing+lantern&duration=5)",
        ].join("\n\n"),
      },
    });
    const video = wrapper.get("figure.ui-md-video video");
    expect(video.attributes("src")).toBe("asset://localhost/%2Ftmp%2Fstarlight.mp4");
    expect(video.attributes("controls")).toBeDefined();
    expect(video.attributes("playsinline")).toBeDefined();
    expect(wrapper.get("figure.ui-md-video figcaption").text()).toBe("星夜动画");

    const pending = wrapper.findAll("figure.ui-md-media-pending");
    expect(pending).toHaveLength(2);
    expect(pending[0]?.attributes("data-media-kind")).toBe("image");
    expect(pending[0]?.text()).toContain("小狐狸提灯");
    expect(pending[1]?.attributes("data-media-kind")).toBe("video");
    expect(pending[1]?.text()).toContain("灯笼微光");
    expect(wrapper.find("img").exists()).toBe(false);
    expect(wrapper.find(".ui-image-group").exists()).toBe(false);
  });
});
