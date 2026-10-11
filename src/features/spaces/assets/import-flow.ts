import { useSpaceStore } from "../session/space-store";
import { chooseSpaceSourcePath } from "./asset-operations";
import { spaceImportContext } from "./import-context";
import type { SpaceImportSessionLabels } from "./import-contract";
import { importSpaceSource } from "./import-session";

export async function importSpaceFromPicker(labels: SpaceImportSessionLabels) {
  const store = useSpaceStore();
  const context = spaceImportContext();
  const source = await chooseSpaceSourcePath();
  if (!source) return [];
  const spaceId = store.activeSpaceId;
  if (!spaceId) throw new Error("A Space must be active to import files");
  return importSpaceSource({
    context,
    spaceId,
    source,
    position: store.nextInsertionPoint(),
    labels,
  });
}
