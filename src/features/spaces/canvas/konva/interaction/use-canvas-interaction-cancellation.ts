import type { Ref } from "vue";
import { watch } from "vue";

interface InteractionCancellationOptions {
  cancelConnector(): void;
  cancelGesture(): boolean;
  cancelNodeDrag(): void;
  cancelPointer(): void;
  cancelStructurePreview(): void;
  clearSelection(): void;
  connectorDraft: Readonly<Ref<unknown>>;
  directManipulation: Readonly<Ref<unknown>>;
  enterableContentId: Ref<string | undefined>;
  exitGroup(): void;
  groupScopePath(): readonly string[];
  interactionEnabled(): boolean;
  marquee: Readonly<Ref<unknown>>;
  panning: Readonly<Ref<boolean>>;
  tool(): unknown;
  directManipulationChanged(active: boolean): void;
}

export function useCanvasInteractionCancellation(options: InteractionCancellationOptions) {
  watch(() => options.directManipulation.value !== undefined, options.directManipulationChanged);
  watch([options.tool, options.interactionEnabled], resetTransientInteraction);

  function resetTransientInteraction() {
    options.cancelPointer();
    options.cancelConnector();
    options.cancelStructurePreview();
    options.enterableContentId.value = undefined;
  }

  function cancelInteraction() {
    if (options.enterableContentId.value) {
      options.enterableContentId.value = undefined;
      return;
    }
    if (options.connectorDraft.value) return options.cancelConnector();
    if (options.marquee.value || options.panning.value) return options.cancelPointer();
    options.cancelNodeDrag();
    options.cancelStructurePreview();
    if (options.cancelGesture()) return;
    if (options.groupScopePath().length) return options.exitGroup();
    options.clearSelection();
  }

  return { cancelInteraction };
}
