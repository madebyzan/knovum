import { computed, shallowRef, type Ref } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import type { CanvasHostEvents } from "../contracts/canvas-editor";
import type { CanvasPoint } from "../contracts/canvas-editor-port";
import type { StructureScene } from "../document/structure-scene";
import { useSectionDropPreview } from "./use-section-drop-preview";

interface StructureCanvasInteractionOptions {
  events: CanvasHostEvents;
  clearSelection(): void;
  setStructureSelection(structureNodeIds: readonly string[]): void;
  scene: Readonly<Ref<StructureScene>>;
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
  function previewDrop(rootIds: readonly string[], delta: CanvasPoint | undefined) {
    if (delta) dropPreview.update(rootIds, delta);
    else dropPreview.clear();
  }

  async function collapse(structureNodeId: string, collapsed: boolean) {
    try {
      await options.events.onExecuteSpaceEdit({ kind: "section.set-collapsed", structureNodeId, collapsed });
      if (collapsed) options.setStructureSelection([structureNodeId]);
    } catch (error) {
      options.events.onError(error);
    }
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
    clearExpansionPreview: dropPreview.clear,
    previewDrop,
    renameTarget,
    renameError,
  };
}
