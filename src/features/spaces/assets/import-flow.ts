import type { useSpaceStore } from "../session/space-store";
import { chooseSpaceSourcePath } from "./asset-operations";
import { spaceImportContext } from "./import-context";
import { importSpaceSource, type SpaceImportSessionLabels } from "./import-session";

type SpaceStore = ReturnType<typeof useSpaceStore>;

export async function importSpaceFromPicker(store: SpaceStore, labels: SpaceImportSessionLabels) {
  const source = await chooseSpaceSourcePath();
  if (!source) return [];
  const spaceId = store.activeSpaceId;
  if (!spaceId) throw new Error("A Space must be active to import files");
  return importSpaceSource({
    context: spaceImportContext(store),
    spaceId,
    source,
    position: store.nextInsertionPoint(),
    labels,
  });
}
