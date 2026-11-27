import { mergeAttributes, Node, wrappingInputRule, type JSONContent, type MarkdownToken } from "@tiptap/core";
import { TextSelection } from "@tiptap/pm/state";
import { VueNodeViewRenderer } from "@tiptap/vue-3";
import type { Component } from "vue";

export const CALLOUT_KINDS = ["note", "tip", "important", "warning", "caution"] as const;
export type CalloutKind = (typeof CALLOUT_KINDS)[number];

const CALLOUT_KIND_SET = new Set<string>(CALLOUT_KINDS);
const GFM_ALERT_START = /^>[ \t]*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]/im;
const GFM_ALERT_BLOCK =
  /^>[ \t]*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\][ \t]*\r?\n((?:>[ \t]?.*(?:\r?\n|$))+)/i;
const CALLOUT_INPUT_RULE = /^>\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s$/i;

export function normalizeCalloutKind(value: unknown): CalloutKind {
  if (typeof value === "string") {
    const normalized = value.trim().toLowerCase();
    if (CALLOUT_KIND_SET.has(normalized)) return normalized as CalloutKind;
  }
  return "note";
}

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    callout: {
      setCallout: (kind?: CalloutKind) => ReturnType;
      toggleCallout: (kind?: CalloutKind) => ReturnType;
      setCalloutKind: (kind: CalloutKind) => ReturnType;
    };
  }
}

export function createCalloutExtension(interactive: boolean, nodeView?: Component) {
  return Node.create({
    name: "callout",
    group: "block",
    content: "block+",
    defining: true,

    addAttributes() {
      return {
        kind: {
          default: "note",
          parseHTML: (element) => normalizeCalloutKind(element.getAttribute("data-callout-kind")),
          renderHTML: (attributes) => ({ "data-callout-kind": normalizeCalloutKind(attributes.kind) }),
        },
      };
    },

    parseHTML() {
      return [{ tag: 'div[data-type="callout"]' }];
    },

    renderHTML({ HTMLAttributes }) {
      return ["div", mergeAttributes({ "data-type": "callout" }, HTMLAttributes), 0];
    },

    addCommands() {
      return {
        setCallout:
          (kind: CalloutKind = "note") =>
          ({ commands }) =>
            commands.wrapIn(this.name, { kind }),
        toggleCallout:
          (kind: CalloutKind = "note") =>
          ({ commands, editor }) => {
            if (editor.isActive(this.name, { kind })) return commands.lift(this.name);
            if (editor.isActive(this.name)) return commands.updateAttributes(this.name, { kind });
            return commands.wrapIn(this.name, { kind });
          },
        setCalloutKind:
          (kind: CalloutKind) =>
          ({ commands }) =>
            commands.updateAttributes(this.name, { kind }),
      };
    },

    addInputRules() {
      return [
        wrappingInputRule({
          find: CALLOUT_INPUT_RULE,
          type: this.type,
          getAttributes: (match) => ({ kind: normalizeCalloutKind(match[1]) }),
        }),
      ];
    },

    addKeyboardShortcuts() {
      return {
        Backspace: () => {
          const { empty, $anchor } = this.editor.state.selection;
          if (!empty || !this.editor.isActive(this.name)) return false;
          if ($anchor.parentOffset === 0 && $anchor.index($anchor.depth - 1) === 0) {
            return this.editor.commands.lift(this.name);
          }
          return false;
        },
        Enter: () => {
          const { state, view } = this.editor;
          const { empty, $from } = state.selection;
          if (!empty || $from.depth < 2) return false;
          const parent = $from.parent;
          const container = $from.node($from.depth - 1);
          if (container.type.name !== this.name || parent.type.name !== "paragraph") return false;
          if (parent.content.size > 0 || container.childCount < 2) return false;
          if ($from.index($from.depth - 1) !== container.childCount - 1) return false;
          const paragraph = state.schema.nodes.paragraph;
          if (!paragraph) return false;
          const afterCallout = $from.after($from.depth - 1);
          const tr = state.tr
            .delete($from.before($from.depth), $from.after($from.depth))
            .insert(afterCallout - parent.nodeSize, paragraph.create());
          const target = Math.min(afterCallout - parent.nodeSize + 1, tr.doc.content.size);
          view.dispatch(tr.setSelection(TextSelection.near(tr.doc.resolve(target))).scrollIntoView());
          return true;
        },
      };
    },

    markdownTokenizer: {
      name: "callout",
      level: "block",
      start(src: string) {
        return src.match(GFM_ALERT_START)?.index ?? -1;
      },
      tokenize(src, _tokens, lexer) {
        const match = GFM_ALERT_BLOCK.exec(src);
        if (!match) return undefined;
        const kind = normalizeCalloutKind(match[1]);
        const body = (match[2] ?? "").replace(/^>[ \t]?/gm, "").trimEnd();
        const tokens = lexer.blockTokens(body);
        return {
          type: "callout",
          raw: match[0],
          kind,
          tokens,
        } as MarkdownToken;
      },
    },

    parseMarkdown(token, helpers) {
      const kind = normalizeCalloutKind((token as MarkdownToken & { kind?: unknown }).kind);
      const children = helpers.parseChildren(token.tokens ?? []);
      return helpers.createNode(
        "callout",
        { kind },
        children.length ? children : [helpers.createNode("paragraph")],
      );
    },

    renderMarkdown(node: JSONContent, helpers) {
      const kind = normalizeCalloutKind(node.attrs?.kind).toUpperCase();
      const body = helpers.renderChildren(node.content ?? [], "\n\n").trim();
      const quoted = body
        .split("\n")
        .map((line) => (line ? `> ${line}` : ">"))
        .join("\n");
      return `> [!${kind}]\n${quoted}`;
    },

    ...(interactive && nodeView ? { addNodeView: () => VueNodeViewRenderer(nodeView) } : {}),
  });
}
