import { open } from "@tauri-apps/plugin-dialog";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { nodeExportFormats } from "./formats";
import { safeExportFileName } from "./naming";
import { payloadExports } from "./payloads";
import { DEFAULT_EXPORT_FORMAT, selectionExportFormats } from "./selection-formats";
import type { ExportTarget } from "./target";

export function selectionExportTarget(spaceId: string, nodes: readonly SpaceCanvasNode[]): ExportTarget {
  const snapshot = [...nodes];
  const readOffers = () => Promise.all(snapshot.map((node) => nodeExportFormats(spaceId, node)));
  return {
    key: `selection:${spaceId}:${JSON.stringify(snapshot.map(({ id }) => id))}`,
    formats: async () =>
      selectionExportFormats(
        snapshot.map(({ data }) => data.kind),
        await readOffers(),
      ),
    save: async (format, onStart) => {
      const offers = await readOffers();
      const admitted = selectionExportFormats(
        snapshot.map(({ data }) => data.kind),
        offers,
      );
      if (!admitted.formats.includes(format)) throw new Error("The selection cannot use this export format");
      const members = snapshot.map((node, index) => {
        const extension = format === DEFAULT_EXPORT_FORMAT ? offers[index]!.formats[0]! : format;
        const payload = payloadExports(node.data).find((item) => item.extension === extension);
        return {
          nodeId: node.id,
          fileName: `${safeExportFileName(node.data.title) || node.data.kind}.${extension}`,
          text: payload?.text(),
        };
      });
      const directoryPath = await open({ directory: true, multiple: false });
      if (!directoryPath) return false;
      const jobId = crypto.randomUUID();
      onStart(jobId);
      await invokeCommand<void>("plugin:spaces|export_selection", { spaceId, directoryPath, members, jobId });
      return true;
    },
  };
}
