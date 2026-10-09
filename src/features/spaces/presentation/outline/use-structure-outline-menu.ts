import type { ComputedRef } from "vue";
import { useI18n } from "vue-i18n";

import { getCanvasEditorSession } from "../../canvas/editor-session";
import { revealStructureNode } from "../../canvas/structure-navigation";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { ancestorsOf, type StructureIndex } from "../../model/structure/structure-index";
import { isStructureNodeEffectivelyLocked } from "../../model/structure/structure-queries";
import { useSpaceStore } from "../../session/space-store";
import { structureParentTargets } from "../../structure/structure-capabilities";
import { popupSpaceContextMenu, spaceMenuItem } from "../../workspace/space-menu";

interface StructureOutlineMenuOptions {
  index: ComputedRef<StructureIndex>;
  rename(structureNodeId: string): void;
}

export function useStructureOutlineMenu(options: StructureOutlineMenuOptions) {
  const store = useSpaceStore();
  const { t } = useI18n();
  const editor = getCanvasEditorSession();
  const operations = useNodeOperationGate();

  async function show(structureNodeId: string) {
    if (!operations.allows("structure")) return;
    const index = options.index.value;
    const structureNode = index.structureById.get(structureNodeId);
    if (!structureNode) return;
    if (structureNode.kind === "content") editor.selectNodes([structureNode.nodeId]);
    else editor.selectStructureNodes([structureNodeId]);
    const effectivelyLocked = isStructureNodeEffectivelyLocked(structureNodeId, index);
    const primary = [
      spaceMenuItem(t("spaces.canvas.revealInCanvas"), () =>
        revealStructureNode(store, editor, structureNodeId),
      ),
      spaceMenuItem(t("common.rename"), () => options.rename(structureNodeId), {
        enabled: !effectivelyLocked,
      }),
    ];
    const layout = store.editSnapshot.layouts.find(
      (candidate) => candidate.structureNodeId === structureNodeId,
    );
    const containerActions = containerMenuEntries(
      structureNodeId,
      structureNode.kind,
      layout,
      effectivelyLocked,
    );
    const parentTargets = structureParentTargets(
      store.editSnapshot,
      [structureNodeId],
      t("spaces.canvas.spaceRoot"),
    );
    const move = parentTargets.length
      ? [
          {
            text: t("spaces.canvas.moveToContainer"),
            items: parentTargets.map((target) =>
              spaceMenuItem(target.title, () =>
                execute({
                  kind: "structure.reparent",
                  rootIds: [structureNodeId],
                  parentId: target.id,
                  order: (index.childrenByParent.get(target.id) ?? []).length,
                }),
              ),
            ),
          },
        ]
      : [];
    const hasLockedAncestor = ancestorsOf(structureNodeId, index).some(({ locked }) => locked);
    const lock = [
      spaceMenuItem(
        structureNode.locked ? t("spaces.canvas.unlock") : t("spaces.canvas.lock"),
        () =>
          execute({ kind: "structure.set-lock", rootIds: [structureNodeId], locked: !structureNode.locked }),
        { enabled: !hasLockedAncestor },
      ),
    ];
    await popupSpaceContextMenu([primary, containerActions, move, lock]);
  }

  function containerMenuEntries(
    structureNodeId: string,
    kind: "content" | "group" | "section",
    layout: SpaceEditSnapshot["layouts"][number] | undefined,
    locked: boolean,
  ) {
    if (kind === "section" && layout?.kind === "section") {
      return [
        spaceMenuItem(
          t(layout.collapsed ? "spaces.canvas.sections.expand" : "spaces.canvas.sections.collapse"),
          () =>
            execute({
              kind: "section.set-collapsed",
              structureNodeId: structureNodeId,
              collapsed: !layout.collapsed,
            }),
        ),
        spaceMenuItem(
          t("spaces.canvas.sections.fitToContents"),
          () => execute({ kind: "section.fit", structureNodeId: structureNodeId }),
          { enabled: !locked },
        ),
        spaceMenuItem(
          t("spaces.canvas.sections.removeOnly"),
          () => execute({ kind: "section.dissolve", structureNodeId: structureNodeId }),
          { enabled: !locked },
        ),
      ];
    }
    if (kind === "group") {
      return [
        spaceMenuItem(
          t("spaces.canvas.groups.ungroup"),
          () => execute({ kind: "group.dissolve", rootIds: [structureNodeId] }),
          { enabled: !locked },
        ),
      ];
    }
    return [];
  }

  async function execute(command: Parameters<typeof store.executeSpaceEdit>[0]) {
    if (!operations.allows("structure")) return;
    await store.executeSpaceEdit(command);
  }

  return { show };
}
