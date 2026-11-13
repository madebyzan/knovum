import type { Ref } from "vue";

import type { CanvasRect } from "../../canvas/contracts/canvas-editor-port";
import type { ImageExecutionTarget } from "../../contracts/generated/ImageExecutionTarget";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { CreationIntentId } from "../catalog/creation-intent-definitions";
import type { CreationSource } from "../catalog/creation-source";
import type { GeneratorKind } from "../generators/generator-kinds";
import { useCanvasGeneratorStore } from "../generators/generator-store";
import { createImageCutout } from "../image-cutout";
import { deriveImageSet } from "../image-sets";
import { addVideoSubtitles } from "../subtitled-video";

interface DerivationActionsOptions {
  canRun(actionId: CreationIntentId): boolean;
  single: Ref<SpaceCanvasNode | undefined>;
  sources: Ref<readonly CreationSource[]>;
  /** Where the selection is, so a generator opened from it stands beside it. */
  bounds: Ref<CanvasRect | undefined>;
}

export function createDerivationActions({ canRun, single, sources, bounds }: DerivationActionsOptions) {
  const generators = useCanvasGeneratorStore();

  /** Creating from a selection is a generator beside it, with the selection as its references. */
  function generate(kind: GeneratorKind) {
    if (bounds.value)
      generators.open(
        kind,
        { beside: bounds.value },
        sources.value.map(({ nodeId }) => nodeId),
      );
  }

  const handlerByIntent: Record<CreationIntentId, (execution?: ImageExecutionTarget) => void> = {
    "remove-image-background": (execution) =>
      void (single.value && createImageCutout(single.value, execution)),
    "split-image-layers": () => void (single.value && deriveImageSet("split-image-layers", single.value)),
    "slice-sticker-sheet": () => void (single.value && deriveImageSet("slice-sticker-sheet", single.value)),
    "add-video-subtitles": () => void (single.value && addVideoSubtitles(single.value)),
    "draft-document": () => generate("document"),
    "generate-visualization": () => generate("image"),
    "generate-video": () => generate("video"),
  };

  function run(actionId: CreationIntentId, execution?: ImageExecutionTarget) {
    if (canRun(actionId)) handlerByIntent[actionId](execution);
  }

  return { run };
}
