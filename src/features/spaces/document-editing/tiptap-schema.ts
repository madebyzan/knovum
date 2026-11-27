import { getSchema } from "@tiptap/core";
import { Highlight } from "@tiptap/extension-highlight";
import { TaskItem, TaskList } from "@tiptap/extension-list";
import { Subscript } from "@tiptap/extension-subscript";
import { Superscript } from "@tiptap/extension-superscript";
import { TableKit } from "@tiptap/extension-table";
import { TextAlign } from "@tiptap/extension-text-align";
import { Color, FontSize, TextStyle } from "@tiptap/extension-text-style";
import { Typography } from "@tiptap/extension-typography";
import type { Schema } from "@tiptap/pm/model";
import { StarterKit } from "@tiptap/starter-kit";
import type { Component } from "vue";

import { createCalloutExtension } from "./tiptap-callout";
import { createSpaceCodeBlockExtension } from "./tiptap-code-block";
import { createImageGroupExtension } from "./tiptap-image-group";
import { createSpaceImageExtension } from "./tiptap-images";
import { createSpaceMathExtensions, type MathEditTarget } from "./tiptap-math";

export interface SpaceTiptapNodeViews {
  readonly codeBlock?: Component;
  readonly callout?: Component;
  readonly image?: Component;
  readonly imageGroup?: Component;
}

export interface RichTextSchemaOptions {
  readonly includeColor: boolean;
  readonly interactiveSurface: boolean;
  readonly nodeViews?: SpaceTiptapNodeViews | undefined;
  readonly onEditMath?: ((target: MathEditTarget) => void) | undefined;
  readonly mathPlaceholders?: { readonly block?: string; readonly inline?: string } | undefined;
}

/**
 * Shared rich-text schema across the document tab editor, canvas inline editor, and canvas HTML
 * preview renderer so content saved in any surface always round-trips through every other surface.
 */
export function createRichTextSchemaExtensions(options: RichTextSchemaOptions) {
  return [
    StarterKit.configure({
      codeBlock: false,
      dropcursor: { color: "var(--color-fg-accent)", width: 2 },
      heading: { levels: [1, 2, 3, 4, 5, 6] },
      link: { openOnClick: false, defaultProtocol: "https" },
    }),
    createSpaceCodeBlockExtension(options.interactiveSurface, options.nodeViews?.codeBlock),
    createCalloutExtension(options.interactiveSurface, options.nodeViews?.callout),
    ...createSpaceMathExtensions(
      options.interactiveSurface ? options.onEditMath : undefined,
      options.mathPlaceholders,
    ),
    TaskList,
    TaskItem.configure({ nested: true }),
    Highlight.configure({ multicolor: true }),
    createSpaceImageExtension(options.interactiveSurface, options.nodeViews?.image),
    createImageGroupExtension(options.interactiveSurface, options.nodeViews?.imageGroup),
    Subscript,
    Superscript,
    TextStyle,
    ...(options.includeColor ? [Color] : []),
    FontSize,
    TextAlign.configure({ types: ["heading", "paragraph"] }),
    Typography,
    TableKit.configure(
      options.interactiveSurface
        ? { table: { resizable: true, lastColumnResizable: false, cellMinWidth: 72 } }
        : {},
    ),
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
