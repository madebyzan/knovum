import { CodeBlock } from "@tiptap/extension-code-block";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import type { EditorView } from "@tiptap/pm/view";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import { VueNodeViewRenderer } from "@tiptap/vue-3";
import type { Component } from "vue";

import { highlightCodeSpans, type CodeTokenSpan } from "../reading/code-highlighter";
import { handleCodeBlockIndent } from "./tiptap-code-block-indent";

const SHIKI_PLUGIN_KEY = new PluginKey<DecorationSet>("codeBlockShiki");

function isDarkAppearance(): boolean {
  if (typeof document === "undefined") return false;
  return document.documentElement.dataset.theme === "dark";
}

function collectCodeBlocks(doc: Parameters<typeof DecorationSet.create>[0]) {
  const blocks: Array<{ pos: number; language: string; code: string }> = [];
  doc.descendants((node, pos) => {
    if (node.type.name !== "codeBlock") return true;
    const language = typeof node.attrs.language === "string" ? node.attrs.language.trim() : "";
    if (language && node.textContent) blocks.push({ pos, language, code: node.textContent });
    return false;
  });
  return blocks;
}

function buildDecorations(
  doc: Parameters<typeof DecorationSet.create>[0],
  spansByKey: ReadonlyMap<string, readonly CodeTokenSpan[]>,
  dark: boolean,
): DecorationSet {
  const decorations: Decoration[] = [];
  for (const block of collectCodeBlocks(doc)) {
    const key = `${dark ? 1 : 0}:${block.language}\0${block.code}`;
    const spans = spansByKey.get(key);
    if (!spans?.length) continue;
    const contentStart = block.pos + 1;
    const contentEnd = contentStart + block.code.length;
    for (const span of spans) {
      const from = contentStart + span.start;
      const to = Math.min(contentStart + span.end, contentEnd);
      if (to > from) decorations.push(Decoration.inline(from, to, { style: span.style }));
    }
  }
  return DecorationSet.create(doc, decorations);
}

function createShikiHighlightPlugin() {
  const cache = new Map<string, readonly CodeTokenSpan[]>();
  const inflight = new Set<string>();
  let currentDark = isDarkAppearance();

  async function highlightBlock(
    view: EditorView,
    block: { readonly language: string; readonly code: string },
    dark: boolean,
  ) {
    const key = `${dark ? 1 : 0}:${block.language}\0${block.code}`;
    inflight.add(key);
    try {
      const spans = await highlightCodeSpans(block.code, block.language, dark);
      cache.set(key, spans ?? []);
      if (!view.isDestroyed) {
        const next = buildDecorations(view.state.doc, cache, currentDark);
        view.dispatch(view.state.tr.setMeta(SHIKI_PLUGIN_KEY, next));
      }
    } catch {
      cache.set(key, []);
    } finally {
      inflight.delete(key);
    }
  }

  function scheduleHighlights(view: EditorView) {
    const dark = isDarkAppearance();
    currentDark = dark;
    for (const block of collectCodeBlocks(view.state.doc)) {
      const key = `${dark ? 1 : 0}:${block.language}\0${block.code}`;
      if (cache.has(key) || inflight.has(key)) continue;
      void highlightBlock(view, block, dark);
    }
  }

  return new Plugin({
    key: SHIKI_PLUGIN_KEY,
    state: {
      init: (_, state) => buildDecorations(state.doc, cache, currentDark),
      apply(tr, previous) {
        const meta = tr.getMeta(SHIKI_PLUGIN_KEY) as DecorationSet | undefined;
        if (meta) return meta;
        return previous.map(tr.mapping, tr.doc);
      },
    },
    props: {
      decorations: (state) => SHIKI_PLUGIN_KEY.getState(state),
    },
    view(editorView) {
      scheduleHighlights(editorView);
      const observer =
        typeof MutationObserver === "undefined"
          ? undefined
          : new MutationObserver(() => {
              const nextDark = isDarkAppearance();
              if (nextDark === currentDark) return;
              currentDark = nextDark;
              scheduleHighlights(editorView);
              if (!editorView.isDestroyed) {
                const next = buildDecorations(editorView.state.doc, cache, currentDark);
                editorView.dispatch(editorView.state.tr.setMeta(SHIKI_PLUGIN_KEY, next));
              }
            });
      observer?.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
      return {
        update(view, previousState) {
          if (!view.state.doc.eq(previousState.doc)) scheduleHighlights(view);
        },
        destroy() {
          observer?.disconnect();
        },
      };
    },
  });
}

function handleCodeBlockEnter(editor: { state: EditorView["state"]; view: EditorView }): boolean {
  const { $from, empty } = editor.state.selection;
  if (!empty || $from.parent.type.name !== "codeBlock") return false;
  const textBefore = $from.parent.textBetween(0, $from.parentOffset, "\n", "\n");
  const lastNewline = textBefore.lastIndexOf("\n");
  const currentLine = lastNewline === -1 ? textBefore : textBefore.slice(lastNewline + 1);
  const indent = /^[\t ]*/u.exec(currentLine)?.[0] ?? "";
  if (!indent) return false;
  editor.view.dispatch(editor.state.tr.insertText(`\n${indent}`).scrollIntoView());
  return true;
}

export function createSpaceCodeBlockExtension(interactive: boolean, nodeView?: Component) {
  return CodeBlock.extend({
    addKeyboardShortcuts() {
      return {
        ...this.parent?.(),
        Enter: () => handleCodeBlockEnter(this.editor),
        Tab: () => handleCodeBlockIndent(this.editor.view, false),
        "Shift-Tab": () => handleCodeBlockIndent(this.editor.view, true),
        "Mod-Enter": () => {
          if (!this.editor.isActive("codeBlock")) return false;
          return this.editor.commands.exitCode();
        },
      };
    },
    ...(interactive && nodeView ? { addNodeView: () => VueNodeViewRenderer(nodeView) } : {}),
    addProseMirrorPlugins() {
      const parentPlugins = this.parent?.() ?? [];
      return interactive ? [...parentPlugins, createShikiHighlightPlugin()] : parentPlugins;
    },
  });
}
