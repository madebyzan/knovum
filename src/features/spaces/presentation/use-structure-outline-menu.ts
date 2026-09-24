import type { ComputedRef } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { ancestorsOf, type StructureIndex } from "../model/structure/structure-index";
import { isStructureNodeEffectivelyLocked } from "../model/structure/structure-queries";
import { structureParentTargets } from "../organization/structure-capabilities";
import type { useSpaceStore } from "../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem, spaceSubmenu } from "../workspace/space-menu";

interface StructureOutlineMenuOptions {
  index: ComputedRef<StructureIndex>;
  rename(id: string): void;
  reveal(id: string): void;
  snapshot(): SpaceEditSnapshot;
  store: ReturnType<typeof useSpaceStore>;
}

export function useStructureOutlineMenu(options: StructureOutlineMenuOptions) {
  const { t } = useI18n();
  const editor = useCanvasEditorSession();
  const operations = useNodeOperationGate();

  async function show(id: string) {
    if (!operations.allows("structure")) return;
    const index = options.index.value;
    const structureNode = index.structureById.get(id);
    if (!structureNode) return;
    if (structureNode.kind === "content") editor.selectNodes([structureNode.nodeId]);
    else editor.selectStructureNodes([id]);
    const effectivelyLocked = isStructureNodeEffectivelyLocked(id, index);
    const primary = [
      spaceMenuItem(t("spaces.canvas.revealInCanvas"), () => options.reveal(id)),
      spaceMenuItem(t("spaces.navigation.renameNode"), () => options.rename(id), {
        enabled: !effectivelyLocked,
      }),
    ];
    const layout = options.snapshot().layouts.find((candidate) => candidate.structureNodeId === id);
    const containerActions = containerMenuEntries(id, structureNode.kind, layout, effectivelyLocked);
    const parentTargets = structureParentTargets(options.snapshot(), [id], t("spaces.canvas.spaceRoot"));
    const move = parentTargets.length
      ? [
          spaceSubmenu(
            t("spaces.canvas.moveToContainer"),
            parentTargets.map((target) =>
              spaceMenuItem(target.title, () =>
                execute({
                  kind: "structure.reparent",
                  rootIds: [id],
                  parentId: target.id,
                  order: (index.childrenByParent.get(target.id) ?? []).length,
                }),
              ),
            ),
          ),
        ]
      : [];
    const hasLockedAncestor = ancestorsOf(id, index).some(({ locked }) => locked);
    const lock = [
      spaceMenuItem(
        structureNode.locked ? t("spaces.canvas.unlock") : t("spaces.canvas.lock"),
        () => execute({ kind: "structure.set-lock", rootIds: [id], locked: !structureNode.locked }),
        { enabled: !hasLockedAncestor },
      ),
    ];
    await popupSpaceContextMenu([primary, containerActions, move, lock]);
  }

  function containerMenuEntries(
    id: string,
    kind: "content" | "group" | "section",
    layout: SpaceEditSnapshot["layouts"][number] | undefined,
    locked: boolean,
  ) {
    if (kind === "section" && layout?.kind === "section") {
      return [
        spaceMenuItem(
          t(layout.collapsed ? "spaces.canvas.expandSection" : "spaces.canvas.collapseSection"),
          () => execute({ kind: "section.set-collapsed", structureNodeId: id, collapsed: !layout.collapsed }),
        ),
        spaceMenuItem(
          t("spaces.canvas.fitSectionToContents"),
          () => execute({ kind: "section.fit", structureNodeId: id }),
          { enabled: !locked },
        ),
        spaceMenuItem(
          t("spaces.canvas.removeSectionOnly"),
          () => execute({ kind: "section.dissolve", structureNodeId: id }),
          { enabled: !locked },
        ),
      ];
    }
    if (kind === "group") {
      return [
        spaceMenuItem(
          t("spaces.canvas.ungroupSelection"),
          () => execute({ kind: "group.dissolve", rootIds: [id] }),
          { enabled: !locked },
        ),
      ];
    }
    return [];
  }

  async function execute(command: Parameters<typeof options.store.executeSpaceEdit>[0]) {
    if (!operations.allows("structure")) return;
    await options.store.executeSpaceEdit(command);
  }

  return { show };
}
