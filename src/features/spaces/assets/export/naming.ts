import { i18n } from "../../../../foundation/i18n/instance";
import { truncateUtf16 } from "../../model/text";

export function safeExportFileName(value: string) {
  const sanitized = truncateUtf16([...value].map(safeFileNameCharacter).join("").trim(), 160).replace(
    /[. ]+$/u,
    "",
  );
  return WINDOWS_RESERVED_FILE_NAME.test(sanitized) ? `_${sanitized}` : sanitized;
}

const INVALID_FILE_NAME_CHARACTERS = new Set(["/", "\\", ":", "*", "?", '"', "<", ">", "|"]);
const WINDOWS_RESERVED_FILE_NAME = /^(?:con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/iu;

function safeFileNameCharacter(character: string) {
  const codePoint = character.codePointAt(0)!;
  return codePoint < 32 || INVALID_FILE_NAME_CHARACTERS.has(character) ? "-" : character;
}

/**
 * Names one format in a save dialog. A format the product talks about has a
 * translated name; anything else is named by the extension it already carries.
 */
export function exportFilter(extension: string) {
  const key = `spaces.importExport.export.formats.${extension}`;
  return {
    name: i18n.global.te(key) ? i18n.global.t(key) : extension.toUpperCase(),
    extensions: [extension],
  };
}
