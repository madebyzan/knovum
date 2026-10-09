import { useRafFn } from "@vueuse/core";
import { compact } from "es-toolkit";
import { computed, shallowRef, type Ref } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { effectiveOperationRoots } from "../../model/structure/structure-index";
import { worldTransformFor } from "../../model/structure/world-transform";
import { sectionDropIndex, sectionDropMoveCommand } from "../../structure/section-drop-reparent";
import type { CanvasHostEvents } from "../contracts/canvas-editor";
import type { CanvasRect } from "../contracts/canvas-editor-port";
import type { StructureScene } from "../document/structure-scene";
import type { CanvasGestureUpdate } from "./interaction/canvas-gesture-update";
import { useSectionDropPreview } from "./use-section-drop-preview";

interface StructureCanvasInteractionOptions {
  events: CanvasHostEvents;
  clearSelection(): void;
  setStructureSelection(structureNodeIds: readonly string[]): void;
  scene: Readonly<Ref<StructureScene>>;
  selectedIds: Readonly<Ref<ReadonlySet<string>>>;
  snapshot(): SpaceEditSnapshot;
  temporarilyExpandedSectionId: Ref<string | undefined>;
}

export function useStructureCanvasInteractions(options: StructureCanvasInteractionOptions) {
  const renameStructureNodeId = shallowRef<string>();
  const renameError = shallowRef<string>();
  const dropTargetId = shallowRef<string>();
  const dropPreview = useSectionDropPreview({
    dropTargetId,
    snapshot: options.snapshot,
    temporarilyExpandedSectionId: options.temporarilyExpandedSectionId,
  });
  const renameTarget = computed(() => {
    const structureNodeId = renameStructureNodeId.value;
    if (!structureNodeId) return undefined;
    const container = [...options.scene.value.sections, ...options.scene.value.groups].find(
      (candidate) => ("section" in candidate ? candidate.section.id : candidate.group.id) === structureNodeId,
    );
    if (!container) return undefined;
    const title = "section" in container ? container.section.title : container.group.title;
    return { structureNodeId, title, bounds: container.bounds };
  });
  const report = (operation: Promise<unknown>) => void operation.catch(options.events.onError);

  function gestureRootIds(structureNodeId: string) {
    const selected = options.selectedIds.value;
    return selected.has(structureNodeId) ? [...selected] : [structureNodeId];
  }

  function move(structureNodeId: string, delta: { x: number; y: number }) {
    report(
      options.events.onExecuteSpaceEdit(
        sectionDropMoveCommand(
          { snapshot: options.snapshot(), rootIds: gestureRootIds(structureNodeId), delta },
          options.temporarilyExpandedSectionId.value,
        ),
      ),
    );
  }

  let groupMove: { id: string; delta: { x: number; y: number } } | undefined;
  const groupPreview = useRafFn(
    () => {
      groupPreview.pause();
      if (groupMove) dropPreview.update(gestureRootIds(groupMove.id), groupMove.delta);
    },
    { immediate: false },
  );
  function previewMove(structureNodeId: string, delta: { x: number; y: number } | undefined) {
    if (!delta) {
      groupPreview.pause();
      dropPreview.clear();
      return;
    }
    groupMove = { id: structureNodeId, delta };
    groupPreview.resume();
  }

  function previewContentMove(updates: readonly CanvasGestureUpdate[]) {
    if (!updates.length) {
      dropPreview.clear();
      return;
    }
    if (!options.scene.value.sections.length) return dropPreview.clear();
    const cached = sectionDropIndex(options.snapshot());
    const index = cached.structure;
    const layouts = cached.layouts;
    const structureNodeIds = compact(updates.map(({ nodeId }) => index.nodeIdToStructureNodeId.get(nodeId)));
    const roots = effectiveOperationRoots(structureNodeIds, index).map(({ id }) => id);
    const firstStructureNodeId = index.nodeIdToStructureNodeId.get(updates[0]!.nodeId);
    if (!firstStructureNodeId) return;
    const world = worldTransformFor(firstStructureNodeId, index, layouts);
    const firstLayout = layouts.get(firstStructureNodeId);
    if (
      firstLayout?.kind !== "content" ||
      Math.abs(updates[0]!.frame.width - firstLayout.size.width * world.scale) > 1e-6 ||
      Math.abs(updates[0]!.frame.height - firstLayout.size.height * world.scale) > 1e-6
    ) {
      dropPreview.clear();
      return;
    }
    const delta = { x: updates[0]!.frame.x - world.x, y: updates[0]!.frame.y - world.y };
    dropPreview.update(roots, delta);
  }

  async function collapse(structureNodeId: string, collapsed: boolean) {
    try {
      await options.events.onExecuteSpaceEdit({ kind: "section.set-collapsed", structureNodeId, collapsed });
      if (collapsed) options.setStructureSelection([structureNodeId]);
    } catch (error) {
      options.events.onError(error);
    }
  }

  function resize(structureNodeId: string, frame: CanvasRect) {
    report(options.events.onExecuteSpaceEdit({ kind: "section.resize-frame", structureNodeId, frame }));
  }

  function transform(
    structureNodeId: string,
    input: { anchor: { x: number; y: number }; rotationDelta: number; scaleFactor: number },
  ) {
    report(
      options.events.onExecuteSpaceEdit({
        kind: "structure.transform",
        rootIds: gestureRootIds(structureNodeId),
        anchor: input.anchor,
        translation: { x: 0, y: 0 },
        rotationDelta: input.rotationDelta,
        scaleFactor: input.scaleFactor,
      }),
    );
  }

  async function commitRename(title: string) {
    const target = renameTarget.value;
    if (!target) return;
    if (target.title === title) {
      renameStructureNodeId.value = undefined;
      return;
    }
    try {
      await options.events.onExecuteSpaceEdit({
        kind: "container.rename",
        structureNodeId: target.structureNodeId,
        title,
      });
      renameStructureNodeId.value = undefined;
      renameError.value = undefined;
    } catch (error) {
      renameError.value = getErrorMessage(error);
      options.events.onError(error);
    }
  }

  function enterGroup(structureNodeId: string) {
    options.clearSelection();
    options.events.onEnterGroup(structureNodeId);
  }

  function exitGroup() {
    if (!options.events.onExitGroup()) return;
    options.clearSelection();
  }

  return {
    beginRename: (structureNodeId: string) => {
      renameError.value = undefined;
      renameStructureNodeId.value = structureNodeId;
    },
    cancelRename: () => {
      renameError.value = undefined;
      renameStructureNodeId.value = undefined;
    },
    collapse,
    commitRename,
    enterGroup,
    exitGroup,
    dropTargetId,
    clearExpansionPreview: () => {
      groupPreview.pause();
      dropPreview.clear();
    },
    move,
    previewMove,
    renameTarget,
    renameError,
    resize,
    transform,
    previewContentMove,
  };
}
