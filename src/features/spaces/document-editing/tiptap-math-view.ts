import type { Node as ProseMirrorNode } from "@tiptap/pm/model";
import { render as renderKatex } from "katex";

import type { MathEditTarget } from "./tiptap-math-rules";

export const KATEX_OPTIONS = { throwOnError: false, strict: false } as const;

interface MathRenderHost {
  readonly wrapper: HTMLElement;
  readonly target: HTMLElement;
  readonly kind: "block" | "inline";
  readonly placeholder: string;
}

function renderMathNodeDom(host: MathRenderHost, latex: string) {
  const source = latex.trim();
  host.wrapper.setAttribute("data-latex", latex);
  host.wrapper.dataset.empty = source ? "false" : "true";
  if (!source) {
    host.target.textContent = host.placeholder;
    return;
  }
  try {
    renderKatex(source, host.target, { ...KATEX_OPTIONS, displayMode: host.kind === "block" });
  } catch {
    host.target.textContent = source;
  }
}

export function createLiveMathNodeView(
  kind: "block" | "inline",
  isEditable: () => boolean,
  placeholder: string,
  onEditMath?: (target: MathEditTarget) => void,
) {
  return ({ node, getPos }: { node: ProseMirrorNode; getPos: () => number | undefined }) => {
    let current = node;
    const wrapper = document.createElement(kind === "block" ? "div" : "span");
    wrapper.className = "tiptap-mathematics-render";
    if (onEditMath) wrapper.classList.add("tiptap-mathematics-render--editable");
    wrapper.dataset.type = kind === "block" ? "block-math" : "inline-math";

    const inner = kind === "block" ? document.createElement("div") : wrapper;
    if (kind === "block") {
      inner.className = "block-math-inner";
      wrapper.appendChild(inner);
    }

    const host: MathRenderHost = { wrapper, target: inner, kind, placeholder };
    const paint = (next: ProseMirrorNode) => {
      const latex = typeof next.attrs.latex === "string" ? next.attrs.latex : "";
      renderMathNodeDom(host, latex);
    };

    const handleClick = (event: Event) => {
      if (!onEditMath || !isEditable()) return;
      event.preventDefault();
      event.stopPropagation();
      const pos = getPos();
      if (pos === undefined) return;
      const latex = typeof current.attrs.latex === "string" ? current.attrs.latex : "";
      onEditMath({ kind, pos, latex });
    };

    if (onEditMath) wrapper.addEventListener("click", handleClick);
    paint(current);

    return {
      dom: wrapper,
      update(updatedNode: ProseMirrorNode) {
        if (updatedNode.type !== current.type) return false;
        current = updatedNode;
        paint(updatedNode);
        return true;
      },
      destroy() {
        wrapper.removeEventListener("click", handleClick);
      },
    };
  };
}
