import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

const CITATION_PATTERN = /\[C([1-9]\d*)\]/gu;
const CITATION_PLUGIN_KEY = new PluginKey<DecorationSet>("groundedCitations");

export function createGroundedCitationExtension(openCitation: (index: number) => void) {
  return Extension.create({
    name: "groundedCitations",
    addProseMirrorPlugins() {
      return [
        new Plugin({
          key: CITATION_PLUGIN_KEY,
          state: {
            init: (_, state) => citationDecorations(state.doc),
            apply: (transaction, decorations) =>
              transaction.docChanged
                ? citationDecorations(transaction.doc)
                : decorations.map(transaction.mapping, transaction.doc),
          },
          props: {
            decorations: (state) => CITATION_PLUGIN_KEY.getState(state),
            handleDOMEvents: {
              mousedown: (_view, event) => {
                if (!citationTarget(event.target)) return false;
                event.preventDefault();
                return true;
              },
              click: (_view, event) => {
                const handled = openCitationTarget(event.target, openCitation);
                if (handled) event.preventDefault();
                return handled;
              },
              keydown: (_view, event) => {
                if (event.key !== "Enter" && event.key !== " ") return false;
                const handled = openCitationTarget(event.target, openCitation);
                if (handled) event.preventDefault();
                return handled;
              },
            },
          },
        }),
      ];
    },
  });
}

function citationTarget(target: EventTarget | null): HTMLElement | null {
  return target instanceof HTMLElement ? target.closest<HTMLElement>("[data-citation-index]") : null;
}

function openCitationTarget(target: EventTarget | null, openCitation: (index: number) => void) {
  const citation = citationTarget(target);
  if (!citation) return false;
  const index = Number.parseInt(citation.dataset.citationIndex ?? "", 10);
  if (!Number.isFinite(index)) return false;
  openCitation(index);
  return true;
}

function createCitationBadge(indexText: string): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "ui-citation";
  button.dataset.citationIndex = indexText;
  button.contentEditable = "false";
  button.textContent = indexText;
  return button;
}

function citationDecorations(doc: Parameters<typeof DecorationSet.create>[0]) {
  const decorations: Decoration[] = [];
  doc.descendants((node, position) => {
    if (node.type.name === "codeBlock") return false;
    if (!node.isText || !node.text || node.marks.some((mark) => mark.type.name === "code")) return;
    for (const match of node.text.matchAll(CITATION_PATTERN)) {
      const indexText = match[1];
      if (match.index === undefined || !indexText) continue;
      const from = position + match.index;
      const to = from + match[0].length;
      decorations.push(
        Decoration.inline(from, to, {
          class: "grounded-citation-source",
          "aria-hidden": "true",
        }),
        Decoration.widget(to, () => createCitationBadge(indexText), {
          side: -1,
          key: `citation-${from}-${indexText}`,
        }),
      );
    }
  });
  return DecorationSet.create(doc, decorations);
}
