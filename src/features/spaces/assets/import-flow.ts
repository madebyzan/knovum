import type { useSpaceStore } from "../session/space-store";
import { chooseSpaceSourcePath } from "./asset-operations";
import { spaceImportContext, type SpaceImportStore } from "./import-context";
import type { SpaceImportSessionLabels } from "./import-contract";
import { importSpaceSource } from "./import-session";

type SpaceStore = SpaceImportStore & Pick<ReturnType<typeof useSpaceStore>, "nextInsertionPoint">;

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
