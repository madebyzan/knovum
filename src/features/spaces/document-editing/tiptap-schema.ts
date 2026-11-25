import { convertFileSrc } from "@tauri-apps/api/core";
import { getSchema } from "@tiptap/core";
import { Highlight } from "@tiptap/extension-highlight";
import { Image } from "@tiptap/extension-image";
import { TaskItem, TaskList } from "@tiptap/extension-list";
import { Mathematics } from "@tiptap/extension-mathematics";
import { Subscript } from "@tiptap/extension-subscript";
import { Superscript } from "@tiptap/extension-superscript";
import { TableKit } from "@tiptap/extension-table";
import { TextAlign } from "@tiptap/extension-text-align";
import { Color, FontSize, TextStyle } from "@tiptap/extension-text-style";
import { Typography } from "@tiptap/extension-typography";
import type { Schema } from "@tiptap/pm/model";
import { StarterKit } from "@tiptap/starter-kit";

const SpaceImage = Image.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      assetPath: { default: null, rendered: false },
      src: {
        default: null,
        renderHTML: (attributes) => ({
          src: attributes.assetPath ? convertFileSrc(attributes.assetPath) : attributes.src,
        }),
      },
    };
  },
});

export interface RichTextSchemaOptions {
  readonly includeColor: boolean;
  readonly interactiveSurface: boolean;
}

/**
 * Shared rich-text schema across the document tab editor, canvas inline editor, and canvas HTML
 * preview renderer so content saved in any surface always round-trips through every other surface.
 */
export function createRichTextSchemaExtensions(options: RichTextSchemaOptions) {
  return [
    StarterKit.configure({
      heading: { levels: [1, 2, 3, 4, 5, 6] },
      link: { openOnClick: false, defaultProtocol: "https" },
    }),
    TaskList,
    TaskItem.configure({ nested: true }),
    Highlight.configure({ multicolor: true }),
    SpaceImage.configure({
      allowBase64: false,
      inline: false,
      ...(options.interactiveSurface
        ? {
            resize: {
              enabled: true,
              directions: ["left", "right"],
              minWidth: 80,
              minHeight: 48,
              alwaysPreserveAspectRatio: true,
            },
          }
        : {}),
    }),
    Subscript,
    Superscript,
    TextStyle,
    ...(options.includeColor ? [Color] : []),
    FontSize,
    TextAlign.configure({ types: ["heading", "paragraph"] }),
    Typography,
    TableKit.configure(
      options.interactiveSurface ? { table: { resizable: true, lastColumnResizable: true } } : {},
    ),
    Mathematics.configure({
      katexOptions: {
        throwOnError: false,
        strict: "ignore",
        output: "html",
      },
    }),
  ];
}

export function createCanvasTextSchema(kind: "note" | "text" = "text") {
  return getSchema(
    createRichTextSchemaExtensions({ includeColor: kind === "text", interactiveSurface: false }),
  );
}

const cachedSchemas: Partial<Record<"note" | "text", Schema>> = {};

/** Lazily compiles and caches one shared ProseMirror Schema per canvas rich-text kind. */
export function getCanvasTextSchema(kind: "note" | "text" = "text"): Schema {
  return (cachedSchemas[kind] ??= createCanvasTextSchema(kind));
}
