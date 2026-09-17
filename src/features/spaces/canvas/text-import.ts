import { createSpaceNotePayload } from "../editor/tiptap-content";
import { truncateUtf16 } from "../model/text";

export function noteFromPlainText(value: string, defaultTitle: string) {
  const firstLine = value.split(/\r?\n/u, 1)[0] ?? "";
  return createSpaceNotePayload(truncateUtf16(firstLine, 80) || defaultTitle, value);
}
