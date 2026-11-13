import { localModelChoiceOptions } from "../../../foundation/ai/local-model-choice";
import { decideModelChoice, type ModelChoiceOption } from "../../../foundation/ai/model-choice";
import { queryClient } from "../../../foundation/query/query-client";
import { localModelsQueryOptions } from "../../../foundation/runtime/use-local-models-query";
import type { LocalModelCapability } from "../../../shared/contracts/generated/LocalModelCapability";
import type { ImageLayersModels } from "../contracts/generated/ImageLayersModels";
import { parseImageExecutionTarget, verifiedImageTargetOptions } from "./image-model-targets";
import { getImageOperationCapabilities } from "./image-operation";

const PASSES = [
  { key: "saliency", capability: "imageSegmentation", labelKey: "spaces.creation.image.removeBackground" },
  { key: "selection", capability: "imageSelection", labelKey: "spaces.media.imageEditor.selection.title" },
  { key: "depth", capability: "imageDepth", labelKey: "spaces.media.imageEditor.depth.action" },
  {
    key: "inpainting",
    capability: "imageInpainting",
    labelKey: "spaces.media.imageEditor.removeObject.tool",
  },
] as const satisfies readonly {
  key: keyof ImageLayersModels;
  capability: LocalModelCapability;
  labelKey: string;
}[];

export interface ImageLayersChoiceGroup {
  key: keyof ImageLayersModels;
  labelKey: string;
  options: readonly ModelChoiceOption[];
  selectedId: string;
}

export type ImageLayersDecision =
  | { kind: "run"; models: ImageLayersModels }
  | { kind: "choose"; groups: ImageLayersChoiceGroup[]; resolved: Partial<ImageLayersModels> };

/**
 * Each layer pass chooses its model independently: one verified download runs unasked, several open
 * a choice, and none keeps the Settings preference to be prepared on start. The split freezes all
 * four so a retry replays the same models.
 */
export async function readImageLayersModelChoice(): Promise<ImageLayersDecision> {
  const [capabilities, models] = await Promise.all([
    getImageOperationCapabilities(),
    queryClient.fetchQuery({ ...localModelsQueryOptions, staleTime: 0, retry: false }),
  ]);
  const resolved: Partial<ImageLayersModels> = {};
  const groups: ImageLayersChoiceGroup[] = [];
  for (const pass of PASSES) {
    const targets = capabilities.targets.filter((target) => target.capability === pass.capability);
    const verified = verifiedImageTargetOptions(targets, models, pass.capability);
    const options = localModelChoiceOptions(models, pass.capability).flatMap((option) => {
      const target = verified.find(
        (candidate) => candidate.execution.type === "local" && candidate.execution.model === option.id,
      );
      return target ? [{ ...option, id: JSON.stringify(target.execution) }] : [];
    });
    const decision = decideModelChoice(options);
    if (decision.kind === "choose") {
      groups.push({
        key: pass.key,
        labelKey: pass.labelKey,
        options: decision.options,
        selectedId: decision.selectedId,
      });
      continue;
    }
    if (decision.kind === "run") {
      resolved[pass.key] = parseImageExecutionTarget(decision.id);
      continue;
    }
    const selected = targets.find((target) => target.selected);
    if (!selected) throw new Error("Image layer model is not configured");
    resolved[pass.key] = selected.execution;
  }
  if (groups.length) return { kind: "choose", groups, resolved };
  return { kind: "run", models: resolved as ImageLayersModels };
}
