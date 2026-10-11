import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

const citationPattern = /\[C(\d+)\]/gu;
const citationKey = new PluginKey<DecorationSet>("groundedCitations");

export function createGroundedCitationExtension(openCitation: (index: number) => void) {
  return Extension.create({
    name: "groundedCitations",
    addProseMirrorPlugins() {
      return [
        new Plugin({
          key: citationKey,
          state: {
            init: (_, state) => citationDecorations(state.doc),
            apply: (transaction, decorations) =>
              transaction.docChanged
                ? citationDecorations(transaction.doc)
                : decorations.map(transaction.mapping, transaction.doc),
          },
          props: {
            decorations: (state) => citationKey.getState(state),
            handleClick: (_view, _position, event) => {
              return openCitationTarget(event.target, openCitation);
            },
            handleKeyDown: (_view, event) => {
              if (event.key !== "Enter" && event.key !== " ") return false;
              const handled = openCitationTarget(event.target, openCitation);
              if (handled) event.preventDefault();
              return handled;
            },
          },
        }),
      ];
    },
  });
}

function openCitationTarget(target: EventTarget | null, openCitation: (index: number) => void) {
  if (!(target instanceof HTMLElement)) return false;
  const citation = target.closest<HTMLElement>("[data-citation-index]");
  if (!citation) return false;
  const index = Number.parseInt(citation.dataset.citationIndex ?? "", 10);
  if (!Number.isFinite(index)) return false;
  openCitation(index);
  return true;
}

function citationDecorations(doc: Parameters<typeof DecorationSet.create>[0]) {
  const decorations: Decoration[] = [];
  doc.descendants((node, position) => {
    if (!node.isText || !node.text) return;
    for (const match of node.text.matchAll(citationPattern)) {
      const indexText = match[1];
      if (match.index === undefined || !indexText) continue;
      const from = position + match.index;
      decorations.push(
        Decoration.inline(from, from + match[0].length, {
          class: "grounded-citation",
          "data-citation-index": indexText,
          "data-citation-label": indexText,
          role: "button",
          tabindex: "0",
        }),
      );
    }
  });
  return DecorationSet.create(doc, decorations);
}
