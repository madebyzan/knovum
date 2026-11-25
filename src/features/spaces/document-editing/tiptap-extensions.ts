import { FileHandler } from "@tiptap/extension-file-handler";
import { FindAndReplace } from "@tiptap/extension-find-and-replace";
import { Placeholder } from "@tiptap/extensions";
import { Markdown } from "@tiptap/markdown";

import { createGroundedCitationExtension } from "./tiptap-citations";
import { EditingResponder } from "./tiptap-editing-responder";
import {
  insertSpaceTiptapImages,
  TIPTAP_IMAGE_MIME_TYPES,
  type SpaceTiptapImageEditor,
} from "./tiptap-images";
import { MarkdownPaste } from "./tiptap-markdown-paste";
import { ReasoningTrace } from "./tiptap-reasoning-trace";
import { createCanvasTextSchema, createRichTextSchemaExtensions } from "./tiptap-schema";

type ImageErrorHandler = (error: unknown) => void;

export { createCanvasTextSchema };

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
