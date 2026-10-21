import { save } from "@tauri-apps/plugin-dialog";
import { extname } from "pathe";

import { exportFilter } from "./naming";

/**
 * Asks where to save `name` in a format that was already chosen. The format is the instruction,
 * so the destination only names the file: one that does not carry the chosen extension is given it.
 */
export async function chooseExportDestination(name: string, format: string): Promise<string | undefined> {
  const destination = await save({ defaultPath: `${name}.${format}`, filters: [exportFilter(format)] });
  if (!destination) return undefined;
  return extname(destination).slice(1).toLowerCase() === format ? destination : `${destination}.${format}`;
}
