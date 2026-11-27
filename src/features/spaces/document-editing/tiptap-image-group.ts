import { mergeAttributes, Node, type JSONContent, type MarkdownToken } from "@tiptap/core";
import { TextSelection } from "@tiptap/pm/state";
import { VueNodeViewRenderer } from "@tiptap/vue-3";
import type { Component } from "vue";

import {
  normalizeImageGroupLayout,
  resolveImageGroupColumns,
  type ImageGroupLayout,
} from "../../../shared/ui/image-group-layout";
import { formatDirectiveAttrs, matchDirectiveBlock } from "../../../shared/ui/markdown-directives";
import {
  createImageGroupNormalizerPlugin,
  normalizeImageGroupsInState,
} from "./tiptap-image-group-normalize";

const GALLERY_DIRECTIVE_START = /^:{3,}gallery(?:\{[^}]*\})?[ \t]*$/im;

function isImageGroupInteractiveTarget(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    Boolean(target.closest("input, textarea, button, [data-image-group-interactive]"))
  );
}

function extractImageJsonNodes(nodes: readonly JSONContent[]): JSONContent[] {
  const images: JSONContent[] = [];
  for (const node of nodes) {
    if (node.type === "image") {
      images.push(node);
    } else if (Array.isArray(node.content)) {
      images.push(...extractImageJsonNodes(node.content));
    }
  }
  return images;
}

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    imageGroup: {
      setImageGroupLayout: (layout: ImageGroupLayout) => ReturnType;
    };
  }
}

export function createImageGroupExtension(interactive: boolean, nodeView?: Component) {
  return Node.create({
    name: "imageGroup",
    group: "block",
    content: "image+",
    defining: true,
    isolating: true,
    selectable: true,

    addAttributes() {
      return {
        layout: {
          default: "auto",
          parseHTML: (element) => normalizeImageGroupLayout(element.getAttribute("data-layout")),
          renderHTML: (attributes) => ({ "data-layout": normalizeImageGroupLayout(attributes.layout) }),
        },
      };
    },

    parseHTML() {
      return [{ tag: 'div[data-type="image-group"]' }];
    },

    renderHTML({ node, HTMLAttributes }) {
      const layout = normalizeImageGroupLayout(node.attrs.layout);
      const cols = resolveImageGroupColumns(layout, node.childCount);
      return [
        "div",
        mergeAttributes(
          {
            "data-type": "image-group",
            "data-count": String(node.childCount),
            class: "ui-image-group",
            style: `--image-group-cols:${cols}`,
          },
          HTMLAttributes,
        ),
        ["div", { class: "ui-image-group-grid" }, 0],
      ];
    },

    addCommands() {
      return {
        setImageGroupLayout:
          (layout: ImageGroupLayout) =>
          ({ commands }) =>
            commands.updateAttributes(this.name, { layout: normalizeImageGroupLayout(layout) }),
      };
    },

    addKeyboardShortcuts() {
      return {
        Enter: () => {
          const { state, view } = this.editor;
          const sel = state.selection as {
            node?: { type: { name: string }; nodeSize: number };
            from: number;
          };
          if (sel.node?.type.name !== this.name) return false;
          const paragraph = state.schema.nodes.paragraph;
          if (!paragraph) return false;
          const after = sel.from + sel.node.nodeSize;
          const tr = state.tr.insert(after, paragraph.create());
          tr.setSelection(TextSelection.near(tr.doc.resolve(Math.min(after + 1, tr.doc.content.size))));
          view.dispatch(tr.scrollIntoView());
          return true;
        },
      };
    },

    onCreate() {
      if (!interactive) return;
      const tr = normalizeImageGroupsInState(this.editor.state, this.name, "image");
      if (tr) this.editor.view.dispatch(tr);
    },

    addProseMirrorPlugins() {
      return interactive ? [createImageGroupNormalizerPlugin(this.name, "image")] : [];
    },

    markdownTokenizer: {
      name: "imageGroup",
      level: "block",
      start(src: string) {
        return src.match(GALLERY_DIRECTIVE_START)?.index ?? -1;
      },
      tokenize(src, _tokens, lexer) {
        const matched = matchDirectiveBlock(src, "gallery");
        if (!matched) return undefined;
        const layout = normalizeImageGroupLayout(matched.attrs.layout ?? "gallery");
        const tokens = lexer.blockTokens(matched.body);
        return { type: "imageGroup", raw: matched.raw, layout, tokens } as MarkdownToken;
      },
    },

    parseMarkdown(token, helpers) {
      const layout = normalizeImageGroupLayout((token as MarkdownToken & { layout?: unknown }).layout);
      const parsed = helpers.parseChildren(token.tokens ?? []);
      const images = extractImageJsonNodes(parsed);
      if (!images.length) return helpers.createNode("paragraph");
      return helpers.createNode("imageGroup", { layout }, images);
    },

    renderMarkdown(node: JSONContent, helpers) {
      const layout = normalizeImageGroupLayout(node.attrs?.layout);
      const body = helpers.renderChildren(node.content ?? [], "\n\n").trim();
      if (layout === "auto") return body;
      const attrSuffix = formatDirectiveAttrs({ layout });
      return `:::gallery${attrSuffix}\n${body}\n:::`;
    },

    ...(interactive && nodeView
      ? {
          addNodeView: () =>
            VueNodeViewRenderer(nodeView, {
              stopEvent: ({ event }) => isImageGroupInteractiveTarget(event.target),
              ignoreMutation: () => true,
            }),
        }
      : {}),
  });
}
