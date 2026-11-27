import { describe, expect, it } from "vitest";

import { createMarkdownRenderer } from "../markdown-renderer";
import { hydrateContainerImageGroups } from "../rich-image-group-hydrator";

describe("markdown image groups and compare hydration", () => {
  const md = createMarkdownRenderer();

  it("automatically groups consecutive plain images into an auto image-group grid", () => {
    const html = md.render(
      "![One](https://example.com/1.png)\n\n![Two](https://example.com/2.png)\n\n![Three](https://example.com/3.png)",
    );
    expect(html).toContain('data-type="image-group"');
    expect(html).toContain('data-layout="auto"');
    expect(html).toContain('data-count="3"');
    expect(html).toContain("--image-group-cols:3");
  });

  it("keeps a single standalone image untouched", () => {
    const html = md.render("![Solo](https://example.com/solo.png)");
    expect(html).not.toContain('data-type="image-group"');
    expect(html).toContain("<img");
  });

  it("renders :::gallery{layout=compare|cards|masonry} directives with semantic markup", () => {
    const compareHtml = md.render(
      ':::gallery{layout="compare"}\n![Before](https://example.com/a.png)\n\n![After](https://example.com/b.png)\n:::',
    );
    expect(compareHtml).toContain('data-layout="compare"');
    expect(compareHtml).toContain('data-image-compare="true"');
    expect(compareHtml).toContain("Before");
    expect(compareHtml).toContain("After");

    const cardsHtml = md.render(
      ':::gallery{layout="cards"}\n![Fast | Under 5ms](https://example.com/a.png)\n\n![Safe | Local-first](https://example.com/b.png)\n:::',
    );
    expect(cardsHtml).toContain('data-layout="cards"');
    expect(cardsHtml).toContain("ui-image-group-card-title");
    expect(cardsHtml).toContain("Under 5ms");
  });

  it("hydrates compare slider keyboard controls inside a container", () => {
    const container = document.createElement("div");
    container.innerHTML = md.render(
      ':::gallery{layout="compare"}\n![Before](https://example.com/a.png)\n\n![After](https://example.com/b.png)\n:::',
    );
    hydrateContainerImageGroups(container);
    const stage = container.querySelector<HTMLElement>('[data-image-compare="true"]');
    expect(stage).not.toBeNull();
    stage!.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }));
    expect(stage!.getAttribute("aria-valuenow")).toBe("55");
  });
});
