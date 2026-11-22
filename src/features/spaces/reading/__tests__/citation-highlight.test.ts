import { describe, expect, it, vi } from "vitest";

import { applyProseCitationHighlight, clearCitationHighlights } from "../citation-highlight";

describe("applyProseCitationHighlight", () => {
  it("highlights multi-line Markdown chunk quotes inside rendered HTML and unwraps cleanly on change", () => {
    const container = document.createElement("div");
    container.innerHTML = `
      <h2>Background</h2>
      <p>General intro text.</p>
      <h3>1. 访谈内容</h3>
      <ul>
        <li>Emma Watson 对童年出演哈利·波特的回忆。</li>
        <li>她对自己小时候的外貌和声音的感受。</li>
      </ul>
      <h3>2. 商业观察</h3>
      <p>电影角色如何延伸到周边产品。</p>
    `;
    const scrollSpy = vi.fn<() => void>();
    const originalScrollIntoView = Element.prototype.scrollIntoView;
    Element.prototype.scrollIntoView = scrollSpy;
    try {
      const target = applyProseCitationHighlight(container, {
        quote:
          "### 1. 访谈内容\n- Emma Watson 对童年出演哈利·波特的回忆。\n- 她对自己小时候的外貌和声音的感受。",
      });

      expect(target).not.toBeNull();
      expect(scrollSpy).toHaveBeenCalledTimes(1);
      expect(container.querySelectorAll("[data-reader-citation-hit]").length).toBeGreaterThanOrEqual(2);
      expect(container.querySelectorAll("mark[data-citation-mark]").length).toBe(1);

      const second = applyProseCitationHighlight(container, {
        heading: "2. 商业观察",
        quote: "电影角色如何延伸到周边产品。",
      });
      expect(second?.textContent).toContain("电影角色如何延伸到周边产品。");
      expect(container.querySelectorAll("mark[data-citation-mark]").length).toBe(1);
      expect(container.querySelectorAll("[data-reader-citation-hit]").length).toBe(1);

      clearCitationHighlights(container);
      expect(container.querySelectorAll("mark[data-citation-mark]").length).toBe(0);
      expect(container.querySelectorAll("[data-reader-citation-hit]").length).toBe(0);
    } finally {
      Element.prototype.scrollIntoView = originalScrollIntoView;
    }
  });
});
