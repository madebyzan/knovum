import { localModelChoiceOptions } from "../../../foundation/ai/local-model-choice";
import type { LocalModelAvailability } from "../../../shared/contracts/generated/LocalModelAvailability";
import type { LocalModelCapability } from "../../../shared/contracts/generated/LocalModelCapability";
import type { ImageExecutionTarget } from "../contracts/generated/ImageExecutionTarget";
import type { ImageOperationTargetOption } from "../contracts/generated/ImageOperationTargetOption";

/** Decode a serialized image execution target from a choice option id. */
export function parseImageExecutionTarget(id: string): ImageExecutionTarget {
  return JSON.parse(id) as ImageExecutionTarget;
}

/**
 * The capability's targets that can run now: configuration ready and download verified. Capability
 * availability alone reports configuration, so the catalog decides what is actually installed.
 */
export function verifiedImageTargetOptions(
  targets: readonly ImageOperationTargetOption[],
  models: readonly LocalModelAvailability[],
  capability: LocalModelCapability,
): ImageOperationTargetOption[] {
  const runnable = new Set(
    localModelChoiceOptions(models, capability)
      .filter((option) => option.runnable)
      .map((option) => option.id),
  );
  return targets.filter(
    (target) =>
      target.capability === capability &&
      target.availability.state === "ready" &&
      target.execution.type === "local" &&
      runnable.has(target.execution.model),
  );
}

/** Pick the Settings-selected verified target, falling back to the first verified target. */
export function preferredVerifiedImageTarget(
  verified: readonly ImageOperationTargetOption[],
): ImageOperationTargetOption | undefined {
  return verified.find((option) => option.selected) ?? verified[0];
}
