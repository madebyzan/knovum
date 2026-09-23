import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { truncateUtf16 } from "../model/text";
import { formatExportMarkdown } from "./markdown-document";

export async function exportSpaceAsset(data: SpaceNodePayload): Promise<boolean> {
  if (!isSpaceManagedAssetPayload(data)) throw new Error("This node has no exportable asset");
  const destinationPath = await save({ defaultPath: data.fileName });
  if (!destinationPath) return false;
  await invoke("plugin:spaces|export_asset", { sourcePath: data.assetPath, destinationPath });
  return true;
}

export async function exportSpaceMarkdown(title: string, markdown: string): Promise<boolean> {
  return exportSpaceTextFile({
    defaultPath: `${safeExportFileName(title) || "markdown"}.md`,
    filters: [{ name: "Markdown", extensions: ["md", "markdown"] }],
    text: formatExportMarkdown(title, markdown),
  });
}

export interface SpaceTextExport {
  defaultPath: string;
  filters: { name: string; extensions: string[] }[];
  text: string;
}

export async function exportSpaceTextFile(file: SpaceTextExport): Promise<boolean> {
  const destinationPath = await save({ defaultPath: file.defaultPath, filters: file.filters });
  if (!destinationPath) return false;
  await invoke("plugin:spaces|export_text", {
    destinationPath,
    text: file.text,
  });
  return true;
}

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
