import type { Extension } from "@tiptap/core";
import { FileHandler } from "@tiptap/extension-file-handler";
import { FindAndReplace } from "@tiptap/extension-find-and-replace";
import { Placeholder } from "@tiptap/extensions";
import { Markdown } from "@tiptap/markdown";

import { BlockKeyboardReorder, createEditorErgonomicsExtension } from "./tiptap-block-actions";
import { BlockMenuHighlight } from "./tiptap-block-highlight";
import { createGroundedCitationExtension } from "./tiptap-citations";
import { EditingResponder } from "./tiptap-editing-responder";
import {
  insertSpaceTiptapImages,
  TIPTAP_IMAGE_MIME_TYPES,
  type SpaceTiptapImageEditor,
} from "./tiptap-images";
import { MarkdownPaste } from "./tiptap-markdown-paste";
import type { MathEditTarget } from "./tiptap-math";
import { ReasoningTrace } from "./tiptap-reasoning-trace";
import {
  createCanvasTextSchema,
  createRichTextSchemaExtensions,
  type SpaceTiptapNodeViews,
} from "./tiptap-schema";

type ImageErrorHandler = (error: unknown) => void;

export { createCanvasTextSchema, type SpaceTiptapNodeViews };

export interface SpaceTiptapExtensionExtras {
  readonly nodeViews?: SpaceTiptapNodeViews;
  readonly slashExtension?: Extension;
  readonly onOpenAi?: () => void;
  readonly onFocusTitle?: () => void;
  readonly onEditMath?: (target: MathEditTarget) => void;
  readonly mathPlaceholders?: { readonly block?: string; readonly inline?: string };
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
