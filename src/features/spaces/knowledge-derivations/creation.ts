import { i18n } from "../../../foundation/i18n/instance";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import type { ImageAspectRatio } from "../contracts/generated/ImageAspectRatio";
import type { VideoAspectRatio } from "../contracts/generated/VideoAspectRatio";
import type { CreationSource } from "./catalog/creation-source";
import { startDerivationAction } from "./start-action";

export const VIDEO_DURATION_SECONDS = [5, 10, 15] as const;

export interface CreationDraft {
  output: CreationOutput;
  title: string;
  requirement: string;
  /** Both frames are kept so switching output does not lose a chosen one. */
  imageAspectRatio: ImageAspectRatio;
  videoAspectRatio: VideoAspectRatio;
  durationSeconds: (typeof VIDEO_DURATION_SECONDS)[number];
  target?: TaskTarget | undefined;
}

/**
 * What the user can create from a selection. One entry per output: the Action
 * it starts, the capability its executor needs, and the options that output
 * actually has. Adding an output is adding an entry.
 */
const CREATION_OUTPUTS = {
  document: {
    actionId: "draft-document",
    purpose: "knowledge",
    parameters: ({ title, requirement }: CreationDraft) => ({ title, purpose: requirement }),
  },
  image: {
    actionId: "generate-visualization",
    purpose: "image-generation",
    // The requirement is the whole instruction; the sources supply the look.
    parameters: ({ title, requirement, imageAspectRatio }: CreationDraft) => ({
      title,
      intent: requirement,
      aspectRatio: imageAspectRatio,
    }),
  },
  video: {
    actionId: "generate-video",
    purpose: "video-generation",
    parameters: ({ title, requirement, videoAspectRatio, durationSeconds }: CreationDraft) => ({
      title,
      prompt: requirement,
      aspectRatio: videoAspectRatio,
      durationSeconds,
    }),
  },
} as const;

export type CreationOutput = keyof typeof CREATION_OUTPUTS;
export const creationOutputs = Object.keys(CREATION_OUTPUTS) as CreationOutput[];

export function creationTargetPurpose(output: CreationOutput) {
  return CREATION_OUTPUTS[output].purpose;
}

export function startCreation(sources: readonly CreationSource[], draft: CreationDraft) {
  const output = CREATION_OUTPUTS[draft.output];
  return startDerivationAction({
    sources,
    actionId: output.actionId,
    title: i18n.global.t(`spaces.creation.composer.task.${draft.output}`),
    target: draft.target,
    parameters: output.parameters({
      ...draft,
      title: draft.title.trim(),
      requirement: draft.requirement.trim(),
    }),
  });
}
