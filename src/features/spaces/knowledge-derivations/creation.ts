import { z } from "zod";

import { i18n } from "../../../foundation/i18n/instance";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { CreationSource } from "./catalog/creation-source";
import { startDerivationAction } from "./start-action";

/**
 * What the user can create from a selection. One request shape for all of them:
 * a requirement in words, the selected resources, and the options the output
 * kind actually has.
 */
export const CREATION_OUTPUTS = ["document", "image", "video"] as const;
export type CreationOutput = (typeof CREATION_OUTPUTS)[number];

export const imageAspectRatioSchema = z.enum(["1:1", "3:2", "2:3", "16:9", "9:16"]);
export const videoAspectRatioSchema = z.enum(["16:9", "9:16", "1:1"]);
export const videoDurations = [5, 10, 15] as const;

export interface CreationDraft {
  output: CreationOutput;
  title: string;
  requirement: string;
  imageAspectRatio: z.infer<typeof imageAspectRatioSchema>;
  videoAspectRatio: z.infer<typeof videoAspectRatioSchema>;
  durationSeconds: (typeof videoDurations)[number];
  target?: TaskTargetRef | undefined;
}

const ACTIONS: Record<CreationOutput, string> = {
  document: "draft-document",
  image: "generate-visualization",
  video: "generate-video",
};

export function creationTargetPurpose(output: CreationOutput) {
  return ({ document: "knowledge", image: "image-generation", video: "video-generation" } as const)[output];
}

export function startCreation(sources: readonly CreationSource[], draft: CreationDraft) {
  return startDerivationAction({
    sources,
    actionId: ACTIONS[draft.output],
    title: i18n.global.t(`spaces.creation.composer.task.${draft.output}`),
    target: draft.target,
    parameters: parameters(draft),
  });
}

function parameters(draft: CreationDraft): Record<string, unknown> {
  const title = draft.title.trim();
  const requirement = draft.requirement.trim();
  switch (draft.output) {
    case "document":
      return { title, purpose: requirement };
    case "image":
      // The requirement is the whole instruction; the sources supply the look.
      return { title, intent: requirement, aspectRatio: draft.imageAspectRatio };
    case "video":
      return {
        title,
        prompt: requirement,
        aspectRatio: draft.videoAspectRatio,
        durationSeconds: draft.durationSeconds,
      };
  }
}
