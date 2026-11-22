import { convertFileSrc } from "@tauri-apps/api/core";
import { getSchema } from "@tiptap/core";
import { FileHandler } from "@tiptap/extension-file-handler";
import { FindAndReplace } from "@tiptap/extension-find-and-replace";
import { Highlight } from "@tiptap/extension-highlight";
import { Image } from "@tiptap/extension-image";
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

import { createGroundedCitationExtension } from "./tiptap-citations";
import { EditingResponder } from "./tiptap-editing-responder";
import {
  insertSpaceTiptapImages,
  TIPTAP_IMAGE_MIME_TYPES,
  type SpaceTiptapImageEditor,
} from "./tiptap-images";
import { MarkdownPaste } from "./tiptap-markdown-paste";
import { ReasoningTrace } from "./tiptap-reasoning-trace";

type ImageErrorHandler = (error: unknown) => void;

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

interface RichTextSchemaOptions {
  readonly includeColor: boolean;
  readonly interactiveSurface: boolean;
}

/**
 * Shared rich-text schema across the document tab editor, canvas inline editor, and canvas HTML
 * preview renderer so content saved in any surface always round-trips through every other surface.
 */
function createRichTextSchemaExtensions(options: RichTextSchemaOptions) {
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
) {
  const handleImages = (editor: SpaceTiptapImageEditor, files: File[], position?: number) => {
    void insertSpaceTiptapImages(editor, files, position).catch(onImageError);
  };
  return [
    EditingResponder,
    ...createRichTextSchemaExtensions({ includeColor: false, interactiveSurface: true }),
    FileHandler.configure({
      allowedMimeTypes: TIPTAP_IMAGE_MIME_TYPES,
      consumePasteEvent: true,
      onDrop: (editor, files, position) => handleImages(editor, files, position),
      onPaste: (editor, files) => handleImages(editor, files),
    }),
    ...createMarkdownExtensions(),
    FindAndReplace.configure({ searchDebounceMs: 120, injectCSS: false }),
    Placeholder.configure({ placeholder }),
    ReasoningTrace,
    createGroundedCitationExtension(openCitation),
  ];
}

export function createCanvasTextExtensions(placeholder: string, kind: "note" | "text" = "text") {
  return [
    EditingResponder,
    ...createRichTextSchemaExtensions({ includeColor: kind === "text", interactiveSurface: false }),
    ...createMarkdownExtensions(),
    Placeholder.configure({ placeholder }),
  ];
}

export function createCanvasTextSchema(kind: "note" | "text" = "text") {
  return getSchema(
    createRichTextSchemaExtensions({ includeColor: kind === "text", interactiveSurface: false }),
  );
}
