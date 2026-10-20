import type { Ref, ShallowRef } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { CreationIntentId } from "../catalog/creation-intent-definitions";
import type { CreationSource } from "../catalog/creation-source";
import { startCreation, type CreationDraft, type CreationOutput } from "../creation";
import { createImageCutout } from "../image-cutout";
import { splitImageLayers } from "../image-layers";
import { addVideoSubtitles } from "../subtitled-video";

interface DerivationActionsOptions {
  canRun(actionId: CreationIntentId): boolean;
  canSubmit(): boolean;

  single: Ref<SpaceCanvasNode | undefined>;
  sources: Ref<readonly CreationSource[]>;
  composing: ShallowRef<CreationOutput | undefined>;
}

export function createDerivationActions({
  canRun,
  canSubmit,
  single,
  sources,
  composing,
}: DerivationActionsOptions) {
  const handlerByIntent: Record<CreationIntentId, () => void> = {
    "remove-image-background": () => void (single.value && createImageCutout(single.value)),
    "split-image-layers": () => void (single.value && splitImageLayers(single.value)),
    "add-video-subtitles": () => void (single.value && addVideoSubtitles(single.value)),
    "draft-document": () => (composing.value = "document"),
    "generate-visualization": () => (composing.value = "image"),
    "generate-video": () => (composing.value = "video"),
  };

  function run(actionId: CreationIntentId) {
    if (canRun(actionId)) handlerByIntent[actionId]();
  }

  function submit(draft: CreationDraft) {
    if (!canSubmit() || sources.value.length === 0) return;
    composing.value = undefined;
    void startCreation(sources.value, draft);
  }

  return { run, submit };
}
