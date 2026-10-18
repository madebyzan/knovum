import { invokeCommand } from "../../../../foundation/ipc/invoke-command";

/** What the engine can save this resource as, and why it offers nothing further. */
export interface EngineExportFormats {
  formats: string[];
  unavailable: string | null;
}

/** The formats this resource can be saved as, its own format first. */
export function listExportFormats(spaceId: string, nodeId: string) {
  return invokeCommand<EngineExportFormats>("plugin:spaces|list_export_formats", {
    spaceId,
    nodeId,
  });
}

/** Writes the resource into the destination, in the format the destination names. */
export function exportResource(spaceId: string, nodeId: string, destinationPath: string, jobId: string) {
  return invokeCommand<void>("plugin:spaces|export_resource", {
    spaceId,
    nodeId,
    destinationPath,
    jobId,
  });
}

export function cancelResourceExport(jobId: string) {
  return invokeCommand<boolean>("plugin:spaces|cancel_resource_export", { jobId });
}

/** Saves a managed file by path, for resources that are not canvas nodes. */
export function exportAssetFile(sourcePath: string, destinationPath: string) {
  return invokeCommand<void>("plugin:spaces|export_asset", { sourcePath, destinationPath });
}

export function exportTextFile(destinationPath: string, text: string) {
  return invokeCommand<void>("plugin:spaces|export_text", { destinationPath, text });
}
