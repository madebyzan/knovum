import { convertFileSrc } from "@tauri-apps/api/core";
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
      resize: {
        enabled: true,
        directions: ["left", "right"],
        minWidth: 80,
        minHeight: 48,
        alwaysPreserveAspectRatio: true,
      },
    }),
    FileHandler.configure({
      allowedMimeTypes: TIPTAP_IMAGE_MIME_TYPES,
      consumePasteEvent: true,
      onDrop: (editor, files, position) => handleImages(editor, files, position),
      onPaste: (editor, files) => handleImages(editor, files),
    }),
    Subscript,
    Superscript,
    TextAlign.configure({ types: ["heading", "paragraph"] }),
    Typography,
    TableKit.configure({ table: { resizable: true, lastColumnResizable: true } }),
    Markdown.configure({
      indentation: { style: "space", size: 2 },
      markedOptions: { gfm: true, breaks: false },
    }),
    FindAndReplace.configure({ searchDebounceMs: 120, injectCSS: false }),
    Placeholder.configure({ placeholder }),
    ReasoningTrace,
    createGroundedCitationExtension(openCitation),
  ];
}

export function createCanvasTextExtensions(placeholder: string, kind: "note" | "text" = "text") {
  return [
    EditingResponder,
    StarterKit.configure({
      heading: false,
      codeBlock: false,
      horizontalRule: false,
      link: { openOnClick: false, defaultProtocol: "https" },
    }),
    TaskList,
    TaskItem.configure({ nested: true }),
    TextStyle,
    ...(kind === "text" ? [Color] : []),
    FontSize,
    TextAlign.configure({ types: ["paragraph"] }),
    Typography,
    Placeholder.configure({ placeholder }),
  ];
}
