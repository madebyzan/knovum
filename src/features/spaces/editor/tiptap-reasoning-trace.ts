import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import type { EditorView } from "@tiptap/pm/view";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

const pluginKey = new PluginKey<DecorationSet>("reasoningTrace");

export const ReasoningTrace = Extension.create({
  name: "reasoningTrace",
  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: pluginKey,
        state: {
          init: () => DecorationSet.empty,
          apply(tr, set) {
            return (tr.getMeta(pluginKey) as DecorationSet | undefined) ?? set.map(tr.mapping, tr.doc);
          },
        },
        props: {
          decorations(state) {
            return this.getState(state);
          },
        },
      }),
    ];
  },
});

export function setReasoningTraceRange(view: EditorView, from: number, to: number) {
  const dec = Decoration.inline(from, to, { class: "reasoning-trace" });
  view.dispatch(view.state.tr.setMeta(pluginKey, DecorationSet.create(view.state.doc, [dec])));
}

export function clearReasoningTrace(view: EditorView) {
  view.dispatch(view.state.tr.setMeta(pluginKey, DecorationSet.empty));
}
