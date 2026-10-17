import { save } from "@tauri-apps/plugin-dialog";

import { formatExportMarkdown } from "../markdown-document";
import { exportTextFile } from "./ipc";
import { exportFilter, safeExportFileName } from "./naming";

export interface SpaceTextExport {
  defaultPath: string;
  filters: { name: string; extensions: string[] }[];
  text: string;
}

/** Asks for a destination and writes text the caller already serialized. */
export async function saveSpaceTextFile(file: SpaceTextExport): Promise<boolean> {
  const destinationPath = await save({ defaultPath: file.defaultPath, filters: file.filters });
  if (!destinationPath) return false;
  await exportTextFile(destinationPath, file.text);
  return true;
}

export async function exportSpaceMarkdown(title: string, markdown: string): Promise<boolean> {
  return saveSpaceTextFile({
    defaultPath: `${safeExportFileName(title) || "markdown"}.md`,
    filters: [exportFilter("md")],
    text: formatExportMarkdown(title, markdown),
  });
}
