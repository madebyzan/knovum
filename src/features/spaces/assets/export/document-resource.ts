import { chooseExportDestination } from "./destination";
import { exportDocumentResource, listDocumentResourceExportFormats } from "./ipc";
import { safeExportFileName } from "./naming";
import type { ExportTarget } from "./target";

/** One picture a document carries, as the document and the place it appears name it. */
interface DocumentResourceExport {
  spaceId: string;
  nodeId: string;
  resourceId: string;
  /** The document's title and where the picture appears in it, such as "Report · Page 3". */
  name: string;
}

/** A picture a document carries is saved on its own, like a picture resource. */
export function documentResourceExportTarget(resource: DocumentResourceExport): ExportTarget {
  const { spaceId, nodeId, resourceId } = resource;
  return {
    key: `document-resource:${spaceId}:${nodeId}:${resourceId}`,
    formats: () => listDocumentResourceExportFormats(spaceId, nodeId, resourceId),
    save: async (format) => {
      const path = await chooseExportDestination(safeExportFileName(resource.name) || "image", format);
      if (!path) return false;
      await exportDocumentResource(spaceId, nodeId, resourceId, path);
      return true;
    },
  };
}
