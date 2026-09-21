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
  const destinationPath = await save({
    defaultPath: `${safeFileName(title) || "markdown"}.md`,
    filters: [{ name: "Markdown", extensions: ["md", "markdown"] }],
  });
  if (!destinationPath) return false;
  await invoke("plugin:spaces|export_text", {
    destinationPath,
    text: formatExportMarkdown(title, markdown),
  });
  return true;
}

function safeFileName(value: string) {
  return truncateUtf16([...value].map(safeFileNameCharacter).join("").trim(), 160);
}

const INVALID_FILE_NAME_CHARACTERS = new Set(["/", ":", "*", "?", '"', "<", ">", "|"]);

function safeFileNameCharacter(character: string) {
  const codePoint = character.codePointAt(0)!;
  return codePoint < 32 || INVALID_FILE_NAME_CHARACTERS.has(character) ? "-" : character;
}
