import type { AppCommand } from "../../../shared/contracts/app-command";
import { openSpaceNode } from "../canvas/selection-actions";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { useSpaceStore } from "../session/space-store";

export function spacePreviewCommand(
  store: ReturnType<typeof useSpaceStore>,
  selectedIds: readonly string[],
  title: string,
): AppCommand | undefined {
  const selected = new Set(selectedIds);
  const node = store.nodes.find(
    (candidate) => selected.has(candidate.id) && isSpaceManagedAssetPayload(candidate.data),
  );
  if (!node) return undefined;
  return {
    id: "spaces.system-preview",
    title,
    category: "canvas",
    run: () => openSpaceNode(store, node),
  };
}
