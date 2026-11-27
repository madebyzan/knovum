import { BlockMath, InlineMath } from "@tiptap/extension-mathematics";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet, type EditorView } from "@tiptap/pm/view";

import {
  convertSelectionToInlineMath,
  createBlockMathInputRules,
  createInlineMathInputRules,
  handleBlockMathEnter,
  openSelectedMathNode,
  type MathEditTarget,
} from "./tiptap-math-rules";
import { createLiveMathNodeView, KATEX_OPTIONS } from "./tiptap-math-view";

export type { MathEditTarget } from "./tiptap-math-rules";
export { insertAndEditMath, validateKatexSyntax } from "./tiptap-math-rules";

const MATH_EDITING_KEY = new PluginKey<number | null>("spaceMathEditing");

export function setMathEditingPos(view: EditorView, pos: number | null) {
  if (!view.isDestroyed) view.dispatch(view.state.tr.setMeta(MATH_EDITING_KEY, pos));
}

function createMathEditingPlugin() {
  return new Plugin<number | null>({
    key: MATH_EDITING_KEY,
    state: {
      init: () => null,
      apply(tr, previous) {
        const meta = tr.getMeta(MATH_EDITING_KEY) as number | null | undefined;
        if (meta !== undefined) return meta;
        if (previous === null || !tr.docChanged) return previous;
        const mapped = tr.mapping.mapResult(previous);
        return mapped.deleted ? null : mapped.pos;
      },
    },
    props: {
      decorations(state) {
        const pos = MATH_EDITING_KEY.getState(state);
        if (pos === null || pos === undefined) return DecorationSet.empty;
        const node = state.doc.nodeAt(pos);
        if (!node || (node.type.name !== "blockMath" && node.type.name !== "inlineMath"))
          return DecorationSet.empty;
        return DecorationSet.create(state.doc, [
          Decoration.node(pos, pos + node.nodeSize, { class: "space-math-editing" }),
        ]);
      },
    },
  });
}

export function createSpaceMathExtensions(
  onEditMath?: (target: MathEditTarget) => void,
  placeholders: { readonly block?: string; readonly inline?: string } = {},
) {
  const blockPlaceholder = placeholders.block ?? "Σ  TeX";
  const inlinePlaceholder = placeholders.inline ?? "TeX";

  const SpaceBlockMath = BlockMath.extend({
    renderMarkdown(node) {
      const latex = typeof node.attrs?.latex === "string" ? node.attrs.latex.trim() : "";
      return `$$\n${latex}\n$$`;
    },
    addCommands() {
      return {
        ...this.parent?.(),
        insertBlockMath:
          (options) =>
          ({ commands, editor }) =>
            commands.insertContentAt(options.pos ?? editor.state.selection.from, {
              type: this.name,
              attrs: { latex: options.latex ?? "" },
            }),
      };
    },
    addInputRules() {
      return createBlockMathInputRules(this.type, onEditMath);
    },
    addKeyboardShortcuts() {
      return {
        Enter: () => handleBlockMathEnter(this.editor, onEditMath),
      };
    },
    addProseMirrorPlugins() {
      return [createMathEditingPlugin()];
    },
    addNodeView() {
      return createLiveMathNodeView("block", () => this.editor.isEditable, blockPlaceholder, onEditMath);
    },
  });

  const SpaceInlineMath = InlineMath.extend({
    renderMarkdown(node) {
      const raw = typeof node.attrs?.latex === "string" ? node.attrs.latex : "";
      const latex = raw.trim().replace(/\s*\n\s*/gu, " ");
      return `$${latex}$`;
    },
    addCommands() {
      return {
        ...this.parent?.(),
        insertInlineMath:
          (options) =>
          ({ editor, tr }) => {
            const from = options.pos ?? editor.state.selection.from;
            tr.replaceWith(from, from, this.type.create({ latex: options.latex ?? "" }));
            return true;
          },
      };
    },
    addInputRules() {
      return createInlineMathInputRules(this.type);
    },
    addKeyboardShortcuts() {
      return {
        "Mod-Shift-e": () => convertSelectionToInlineMath(this.editor, onEditMath),
        Enter: () => openSelectedMathNode(this.editor, "inline", onEditMath),
      };
    },
    addNodeView() {
      return createLiveMathNodeView("inline", () => this.editor.isEditable, inlinePlaceholder, onEditMath);
    },
  });

  return [
    SpaceBlockMath.configure({ katexOptions: { ...KATEX_OPTIONS, displayMode: true } }),
    SpaceInlineMath.configure({ katexOptions: { ...KATEX_OPTIONS, displayMode: false } }),
  ];
}
