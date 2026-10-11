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
  isInteractionEnabled(): boolean;
  marquee: Readonly<Ref<unknown>>;
  panning: Readonly<Ref<boolean>>;
  tool(): unknown;
  directManipulationChanged(active: boolean): void;
}

export function useCanvasInteractionCancellation(options: InteractionCancellationOptions) {
  watch(
    () => options.directManipulation.value !== undefined || options.panning.value,
    options.directManipulationChanged,
  );
  watch(options.tool, resetTransientInteraction);
  watch(options.isInteractionEnabled, (enabled) => {
    if (!enabled) suspendInteraction();
  });

  function resetTransientInteraction() {
    options.cancelPointer();
    options.cancelConnector();
    options.cancelStructurePreview();
    options.enterableContentId.value = undefined;
  }

  function suspendInteraction() {
    options.cancelNodeDrag();
    options.cancelGesture();
    resetTransientInteraction();
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
    if (options.cancelGesture()) return options.cancelPointer();
    if (options.groupScopePath().length) return options.exitGroup();
    options.clearSelection();
  }

  return { cancelInteraction, suspendInteraction };
}
