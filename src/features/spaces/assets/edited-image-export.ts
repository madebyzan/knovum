import { save } from "@tauri-apps/plugin-dialog";
import { parse } from "pathe";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
import { safeExportFileName } from "./export";

export async function exportEditedImage(
  spaceId: string,
  nodeId: string,
  data: SpaceImagePayload,
): Promise<boolean> {
  const name = safeExportFileName(parse(data.fileName).name) || "image";
  const destinationPath = await save({
    defaultPath: `${name}.png`,
    filters: [{ name: "PNG", extensions: ["png"] }],
  });
  if (!destinationPath) return false;

  await invoke("plugin:spaces|export_image", {
    spaceId,
    nodeId,
    destinationPath,
  });
  return true;
}
