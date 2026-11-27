import { getSchema, type Extension } from "@tiptap/core";
import { FileHandler } from "@tiptap/extension-file-handler";
import { FindAndReplace } from "@tiptap/extension-find-and-replace";
import { Highlight } from "@tiptap/extension-highlight";
import { TaskItem, TaskList } from "@tiptap/extension-list";
import { Subscript } from "@tiptap/extension-subscript";
import { Superscript } from "@tiptap/extension-superscript";
import { TableKit } from "@tiptap/extension-table";
import { TextAlign } from "@tiptap/extension-text-align";
import { Color, FontSize, TextStyle } from "@tiptap/extension-text-style";
import { Typography } from "@tiptap/extension-typography";
import { Placeholder } from "@tiptap/extensions";
import { Markdown } from "@tiptap/markdown";
import { StarterKit } from "@tiptap/starter-kit";
import type { Component } from "vue";

import { BlockKeyboardReorder, createEditorErgonomicsExtension } from "./tiptap-block-actions";
import { BlockMenuHighlight } from "./tiptap-block-highlight";
import { createCalloutExtension } from "./tiptap-callout";
import { createGroundedCitationExtension } from "./tiptap-citations";
import { createSpaceCodeBlockExtension } from "./tiptap-code-block";
import { EditingResponder } from "./tiptap-editing-responder";
import { createImageGroupExtension } from "./tiptap-image-group";
import {
  createSpaceImageExtension,
  insertSpaceTiptapImages,
  TIPTAP_IMAGE_MIME_TYPES,
  type SpaceTiptapImageEditor,
} from "./tiptap-images";
import { MarkdownPaste } from "./tiptap-markdown-paste";
import { createSpaceMathExtensions, type MathEditTarget } from "./tiptap-math";
import { ReasoningTrace } from "./tiptap-reasoning-trace";

type ImageErrorHandler = (error: unknown) => void;

export interface SpaceTiptapNodeViews {
  readonly codeBlock?: Component;
  readonly callout?: Component;
  readonly image?: Component;
  readonly imageGroup?: Component;
}

export interface SpaceTiptapExtensionExtras {
  readonly nodeViews?: SpaceTiptapNodeViews;
  readonly slashExtension?: Extension;
  readonly onOpenAi?: () => void;
  readonly onFocusTitle?: () => void;
  readonly onEditMath?: (target: MathEditTarget) => void;
  readonly mathPlaceholders?: { readonly block?: string; readonly inline?: string };
}

interface RichTextSchemaOptions {
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
function createRichTextSchemaExtensions(options: RichTextSchemaOptions) {
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

function createMarkdownExtensions() {
  return [
    Markdown.configure({
      indentation: { style: "space", size: 2 },
      markedOptions: { gfm: true, breaks: false },
    }),
    MarkdownPaste,
  ];
}

export function createSpaceTiptapExtensions(
  placeholder: string,
  onImageError: ImageErrorHandler,
  openCitation: (index: number) => void,
  extras: SpaceTiptapExtensionExtras = {},
) {
  const handleImages = (editor: SpaceTiptapImageEditor, files: File[], position?: number) => {
    void insertSpaceTiptapImages(editor, files, position).catch(onImageError);
  };
  return [
    EditingResponder,
    ...createRichTextSchemaExtensions({
      includeColor: false,
      interactiveSurface: true,
      nodeViews: extras.nodeViews,
      onEditMath: extras.onEditMath,
      mathPlaceholders: extras.mathPlaceholders,
    }),
    BlockKeyboardReorder,
    BlockMenuHighlight,
    createEditorErgonomicsExtension(extras.onOpenAi, extras.onFocusTitle),
    ...(extras.slashExtension ? [extras.slashExtension] : []),
    FileHandler.configure({
      allowedMimeTypes: TIPTAP_IMAGE_MIME_TYPES,
      consumePasteEvent: true,
      onDrop: (editor, files, position) => handleImages(editor, files, position),
      onPaste: (editor, files) => handleImages(editor, files),
    }),
    ...createMarkdownExtensions(),
    FindAndReplace.configure({ searchDebounceMs: 120, injectCSS: false }),
    Placeholder.configure({
      showOnlyCurrent: true,
      showOnlyWhenEditable: true,
      placeholder: ({ node }) => {
        if (node.type.name === "heading") return `H${String(node.attrs.level ?? 1)}`;
        if (node.type.name === "paragraph") return placeholder;
        return "";
      },
    }),
    ReasoningTrace,
    createGroundedCitationExtension(openCitation),
  ];
}

export function createCanvasTextExtensions(placeholder: string, kind: "note" | "text" = "text") {
  return [
    EditingResponder,
    ...createRichTextSchemaExtensions({ includeColor: kind === "text", interactiveSurface: false }),
    ...createMarkdownExtensions(),
    Placeholder.configure({
      showOnlyCurrent: true,
      showOnlyWhenEditable: true,
      placeholder: ({ node }) => {
        if (node.type.name === "heading") return `H${String(node.attrs.level ?? 1)}`;
        if (node.type.name === "paragraph") return placeholder;
        return "";
      },
    }),
  ];
}

export function createCanvasTextSchema(kind: "note" | "text" = "text") {
  return getSchema(
    createRichTextSchemaExtensions({ includeColor: kind === "text", interactiveSurface: false }),
  );
}
