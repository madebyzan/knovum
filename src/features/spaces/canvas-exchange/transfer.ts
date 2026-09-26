import { dirname } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";
import { isString } from "es-toolkit";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { CanvasExchangePort } from "./canvas-exchange-port";

const JSON_CANVAS_FILTER = [{ name: "JSON Canvas", extensions: ["canvas"] }];

export async function importJsonCanvasFile(exchange: CanvasExchangePort) {
  const sourcePath = await open({ multiple: false, directory: false, filters: JSON_CANVAS_FILTER });
  if (!isString(sourcePath)) return false;
  await exchange.importJsonCanvas(await readTextFile(sourcePath), await dirname(sourcePath));
  return true;
}

export async function exportJsonCanvasFile(exchange: CanvasExchangePort, title: string) {
  const exported = await exchange.exportJsonCanvas();
  const destinationPath = await save({
    defaultPath: `${canvasFileName(title)}.canvas`,
    filters: JSON_CANVAS_FILTER,
  });
  if (!destinationPath) return false;
  await invoke("plugin:spaces|export_json_canvas_file", {
    request: { destinationPath, source: exported.source, resources: exported.resources },
  });
  return true;
}

function canvasFileName(title: string) {
  const name = title
    .trim()
    .replace(/[\\/:*?"<>|]/gu, "-")
    .replace(/\s+/gu, " ");
  if (!name) throw new Error("Space title cannot be used as a JSON Canvas file name");
  return name;
}
