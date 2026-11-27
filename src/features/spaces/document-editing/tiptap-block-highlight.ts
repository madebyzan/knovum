import { Extension, type Editor } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

interface BlockDecorationTarget {
  readonly pos: number;
  readonly className: string;
}

const BLOCK_HIGHLIGHT_KEY = new PluginKey<DecorationSet>("spaceBlockMenuHighlight");

function buildHighlight(
  doc: Parameters<typeof DecorationSet.create>[0],
  target: BlockDecorationTarget | null,
): DecorationSet {
  if (!target || target.pos < 0 || target.pos >= doc.content.size) return DecorationSet.empty;
  const node = doc.nodeAt(target.pos);
  if (!node) return DecorationSet.empty;
  return DecorationSet.create(doc, [
    Decoration.node(target.pos, target.pos + node.nodeSize, { class: target.className }),
  ]);
}

export function setBlockMenuHighlight(editor: Editor, pos: number | null) {
  if (editor.isDestroyed) return;
  const target = pos === null ? null : { pos, className: "space-block-menu-target" };
  editor.view.dispatch(editor.state.tr.setMeta(BLOCK_HIGHLIGHT_KEY, target));
}

export function setBlockDragSource(editor: Editor, pos: number | null) {
  if (editor.isDestroyed) return;
  const target = pos === null ? null : { pos, className: "space-block-dragging" };
  editor.view.dispatch(editor.state.tr.setMeta(BLOCK_HIGHLIGHT_KEY, target));
}

export const BlockMenuHighlight = Extension.create({
  name: "blockMenuHighlight",
  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: BLOCK_HIGHLIGHT_KEY,
        state: {
          init: () => DecorationSet.empty,
          apply(tr, previous) {
            const meta = tr.getMeta(BLOCK_HIGHLIGHT_KEY) as BlockDecorationTarget | null | undefined;
            if (meta !== undefined) return buildHighlight(tr.doc, meta);
            if (tr.docChanged) return DecorationSet.empty;
            return previous;
          },
        },
        props: {
          decorations: (state) => BLOCK_HIGHLIGHT_KEY.getState(state),
        },
      }),
    ];
  },
});
