import { useSpaceStore } from "../session/space-store";
import { chooseSpaceSourcePath } from "./asset-operations";
import { spaceImportContext } from "./import-context";
import type { SpaceImportSessionLabels } from "./import-contract";
import { importSpaceSource } from "./import-session";

export interface SpaceImportPlacement {
  /** Where the imported resource stands; the next free spot on the canvas when omitted. */
  position?: { x: number; y: number };
  /** Whether the camera brings it into view; off when it lands beside what the person is working on. */
  reveal?: boolean;
}

export async function importSpaceFromPicker(
  labels: SpaceImportSessionLabels,
  placement: SpaceImportPlacement = {},
) {
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
    position: placement.position ?? store.nextInsertionPoint(),
    labels,
    ...(placement.reveal === undefined ? {} : { reveal: placement.reveal }),
  });
}
