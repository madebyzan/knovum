import { availableWhen } from "../../../shared/contracts/action-availability";
import type { AppCommand } from "../../../shared/contracts/app-command";
import type { WorkspaceSidebarPresentation } from "../../../shared/contracts/workspace-sidebar";
import {
  canRenameStructureNode,
  dissolveGroupCapability,
  groupCapability,
} from "../organization/structure-capabilities";
import type { useSpaceStore } from "../session/space-store";

export function createSpaceStructureCommands(
  store: ReturnType<typeof useSpaceStore>,
  selectedStructureNodeIds: () => string[],
  t: (key: string) => string,
  sidebar: WorkspaceSidebarPresentation,
): AppCommand[] {
  return [
    {
      id: "spaces.group-selection",
      title: t("spaces.canvas.groupSelection"),
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
            title: t("spaces.canvas.untitledGroup"),
          })
          .then(() => undefined),
    },
    {
      id: "spaces.dissolve-group",
      title: t("spaces.canvas.ungroupSelection"),
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
      title: t("spaces.navigation.renameNode"),
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
  ];
}
