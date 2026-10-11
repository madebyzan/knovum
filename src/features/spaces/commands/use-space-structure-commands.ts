import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { availableWhen } from "../../../shared/contracts/action-availability";
import type { AppCommand } from "../../../shared/contracts/app-command";
import type { WorkspaceSidebarPresentation } from "../../../shared/contracts/workspace-sidebar";
import { useSpaceStore } from "../session/space-store";
import {
  canRenameStructureNode,
  dissolveGroupCapability,
  groupCapability,
} from "../structure/structure-capabilities";

interface SpaceStructureCommandInput {
  selectedStructureNodeIds: () => string[];

  sidebar: WorkspaceSidebarPresentation;
}

export function useSpaceStructureCommands({ selectedStructureNodeIds, sidebar }: SpaceStructureCommandInput) {
  const store = useSpaceStore();
  const { t } = useI18n();

  return computed<AppCommand[]>(() => [
    {
      id: "spaces.group-selection",
      title: t("spaces.canvas.groups.create"),
      shortcut: "$mod+G",
      category: "canvas",
      availability: () =>
        availableWhen(
          groupCapability(store.editSnapshot, selectedStructureNodeIds()).enabled,
          "selection_not_eligible",
        ),
      run: () =>
        store
          .executeSpaceEdit({
            kind: "group.create",
            groupId: crypto.randomUUID(),
            rootIds: selectedStructureNodeIds(),
            title: t("spaces.canvas.groups.untitled"),
          })
          .then(() => undefined),
    },
    {
      id: "spaces.dissolve-group",
      title: t("spaces.canvas.groups.ungroup"),
      shortcut: "$mod+Shift+G",
      category: "canvas",
      availability: () =>
        availableWhen(
          dissolveGroupCapability(store.editSnapshot, selectedStructureNodeIds()).enabled,
          "selection_not_eligible",
        ),
      run: () =>
        store
          .executeSpaceEdit({ kind: "group.dissolve", rootIds: selectedStructureNodeIds() })
          .then(() => undefined),
    },
    {
      id: "spaces.rename-node",
      title: t("common.rename"),
      shortcut: "F2",
      category: "canvas",
      availability: () =>
        availableWhen(
          canRenameStructureNode(store.editSnapshot, selectedStructureNodeIds()),
          "selection_not_eligible",
        ),
      run: () => {
        sidebar.reveal();
        store.outlineRenameStructureNodeId = selectedStructureNodeIds()[0]!;
      },
    },
  ]);
}
