import { onScopeDispose, watchEffect, type ShallowRef } from "vue";

import {
  createMarkdownRenderer,
  type MarkdownCitations,
  type MarkdownRendererOptions,
} from "./markdown-renderer";
import { createMarkdownStream, type MarkdownFrame } from "./markdown-stream";
import { hydrateRichPreviewContainer } from "./rich-block-hydrator";

interface MarkdownSource {
  content: string;
  citations?: MarkdownCitations | undefined;
}

function hasUnclosedCodeFence(markdown: string): boolean {
  return ((markdown.match(/^[ \t]{0,3}(?:```|~~~)/gmu)?.length ?? 0) & 1) === 1;
}

/**
 * Paints Markdown into `root` block by block. Settled blocks are inserted once and never touched
 * again; only the open last block is replaced when the source grows, so an answer streamed one
 * delta at a time re-renders one block per update, and the blocks above it keep their nodes,
 * selection and layout. Blocks are direct children of the root, exactly where a render of the
 * whole document would place them, so the content stylesheet applies unchanged.
 */
export function useMarkdownPaint(
  root: Readonly<ShallowRef<HTMLElement | null>>,
  source: () => MarkdownSource,
  options: MarkdownRendererOptions = {},
) {
  const md = createMarkdownRenderer(options);
  let citations: MarkdownCitations | undefined;
  let render = createMarkdownStream(md);
  let painted = 0;
  let open: ChildNode[] = [];
  let themeObserver: MutationObserver | undefined;

  function clear(element: HTMLElement) {
    element.replaceChildren();
    painted = 0;
    open = [];
  }

  function ensureThemeObserver() {
    if (themeObserver || typeof MutationObserver === "undefined") return;
    themeObserver = new MutationObserver(() => {
      const element = root.value;
      if (element) void hydrateRichPreviewContainer(element);
    });
    themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
  }

  function paint(element: HTMLElement, frame: MarkdownFrame, unclosedFence: boolean) {
    if (frame.restarted) clear(element);
    for (const node of open) node.remove();
    for (; painted < frame.settled.length; painted += 1)
      element.insertAdjacentHTML("beforeend", frame.settled[painted]!);
    const template = document.createElement("template");
    template.innerHTML = frame.open;
    open = [...template.content.childNodes];
    element.append(template.content);
    const skipMermaidNodes = unclosedFence ? new Set<Node>(open) : undefined;
    void hydrateRichPreviewContainer(element, undefined, skipMermaidNodes);
    const hasHydratableMermaid = [...element.querySelectorAll("pre > code.language-mermaid")].some(
      (code) => !code.parentElement || !skipMermaidNodes?.has(code.parentElement),
    );
    if (hasHydratableMermaid) ensureThemeObserver();
  }

  watchEffect(
    () => {
      const element = root.value;
      const next = source();
      if (!element) return;
      // Citation markers render differently once their sources are known, so every block repaints.
      if (next.citations !== citations) {
        citations = next.citations;
        render = createMarkdownStream(md, citations);
        clear(element);
      }
      paint(element, render(next.content), hasUnclosedCodeFence(next.content));
    },
    { flush: "post" },
  );

  onScopeDispose(() => themeObserver?.disconnect());
}
