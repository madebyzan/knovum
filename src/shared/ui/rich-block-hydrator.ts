import { render as renderKatex } from "katex";

import { renderMermaidDiagram } from "./mermaid-renderer";
import { createMermaidStage, type MermaidStageHandle } from "./mermaid-stage";
import { hydrateContainerImageGroups } from "./rich-image-group-hydrator";

const KATEX_OPTIONS = { throwOnError: false, strict: false } as const;
const mermaidStages = new WeakMap<HTMLElement, MermaidStageHandle>();

function isDarkDocumentTheme(): boolean {
  return typeof document !== "undefined" && document.documentElement.dataset.theme === "dark";
}

/** Hydrates `[data-type="block-math"]` and `[data-type="inline-math"]` elements with KaTeX in place. */
function hydrateContainerMath(container: HTMLElement): void {
  const nodes = container.querySelectorAll<HTMLElement>(
    '[data-type="block-math"][data-latex], [data-type="inline-math"][data-latex]',
  );
  for (const el of nodes) {
    const latex = (el.dataset.latex ?? "").trim();
    if (el.dataset.katexRendered === latex && el.childNodes.length > 0) continue;
    el.dataset.katexRendered = latex;
    if (!latex) {
      el.replaceChildren();
      continue;
    }
    try {
      renderKatex(latex, el, { ...KATEX_OPTIONS, displayMode: el.dataset.type === "block-math" });
    } catch {
      el.textContent = latex;
    }
  }
}

function findMermaidHost(pre: HTMLElement): HTMLElement | null {
  const sibling = pre.nextElementSibling;
  return sibling instanceof HTMLElement && sibling.dataset.mermaidHost === "true" ? sibling : null;
}

function removeMermaidHost(host: HTMLElement | null) {
  if (!host) return;
  mermaidStages.get(host)?.destroy();
  mermaidStages.delete(host);
  host.remove();
}

function attachMermaidHost(
  pre: HTMLElement,
  svgMarkup: string,
  meta: { readonly dark: boolean; readonly themeKey: string; readonly source: string },
) {
  const currentHost = findMermaidHost(pre);
  const host = currentHost ?? document.createElement("div");
  host.dataset.mermaidHost = "true";
  host.className = "ui-mermaid-diagram";
  const compact = Boolean(pre.closest(".canvas-text-prose"));
  let stage = mermaidStages.get(host);
  if (!stage) {
    stage = createMermaidStage(host, { compact });
    mermaidStages.set(host, stage);
  }
  if (!stage.update(svgMarkup, meta.dark)) return;
  if (!currentHost) pre.after(host);
  pre.hidden = true;
  pre.dataset.mermaidTheme = meta.themeKey;
  pre.dataset.mermaidSource = meta.source;
}

function isMermaidUpToDate(pre: HTMLElement, themeKey: string, source: string): boolean {
  return (
    pre.dataset.mermaidTheme === themeKey &&
    pre.dataset.mermaidSource === source &&
    Boolean(findMermaidHost(pre))
  );
}

async function hydrateMermaidCodeBlock(codeElement: HTMLElement, dark: boolean, themeKey: string) {
  const pre = codeElement.parentElement;
  if (!pre?.isConnected) return;
  const source = (codeElement.dataset.rawCode ?? codeElement.textContent ?? "").trim();
  codeElement.dataset.rawCode = source;
  if (!source || isMermaidUpToDate(pre, themeKey, source)) return;
  const rendered = await renderMermaidDiagram(source, dark);
  if (!pre.isConnected) return;
  if (!rendered.ok) {
    pre.hidden = false;
    removeMermaidHost(findMermaidHost(pre));
    return;
  }
  attachMermaidHost(pre, rendered.svg, { dark, themeKey, source });
}

/** Lazily renders `pre > code.language-mermaid` blocks inside a static preview/reading container. */
async function renderContainerMermaidDiagrams(
  container: HTMLElement,
  dark: boolean,
  skipMermaidNodes?: ReadonlySet<Node>,
): Promise<void> {
  for (const orphan of container.querySelectorAll<HTMLElement>('[data-mermaid-host="true"]')) {
    if (orphan.previousElementSibling?.tagName !== "PRE") removeMermaidHost(orphan);
  }
  const codeBlocks = [...container.querySelectorAll<HTMLElement>("pre > code.language-mermaid")].filter(
    (codeElement) => !codeElement.parentElement || !skipMermaidNodes?.has(codeElement.parentElement),
  );
  if (codeBlocks.length === 0) return;
  const themeKey = dark ? "dark" : "light";
  await Promise.all(codeBlocks.map((codeElement) => hydrateMermaidCodeBlock(codeElement, dark, themeKey)));
}

/** Hydrates KaTeX math immediately and lazily renders Mermaid diagrams when present. */
export function hydrateRichPreviewContainer(
  container: HTMLElement,
  dark = isDarkDocumentTheme(),
  skipMermaidNodes?: ReadonlySet<Node>,
): Promise<void> {
  hydrateContainerMath(container);
  hydrateContainerImageGroups(container);
  return renderContainerMermaidDiagrams(container, dark, skipMermaidNodes);
}
