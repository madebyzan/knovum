import type { RoutingPurpose } from "../../../../shared/contracts/generated/RoutingPurpose";
import type { ImageAspectRatio } from "../../contracts/generated/ImageAspectRatio";
import type { VideoAspectRatio } from "../../contracts/generated/VideoAspectRatio";

export const VIDEO_DURATIONS = [5, 10, 15] as const;
export type VideoDuration = (typeof VIDEO_DURATIONS)[number];

/** What a person has asked a generator for. */
export interface GeneratorRequest {
  prompt: string;
  aspectRatio: string;
  durationSeconds: VideoDuration;
}

interface GeneratorKindDefinition {
  /** The Action a generation starts. */
  actionId: string;
  /** Which executors can run it. */
  purpose: RoutingPurpose;
  /** The shapes its output can take, the first by default. One shape needs no choice. */
  aspectRatios: readonly string[];
  /** The lengths its output can run for; empty when it does not play. */
  durations: readonly VideoDuration[];
  parameters(title: string, request: GeneratorRequest): Record<string, unknown>;
}

const IMAGE_ASPECT_RATIOS = [
  "1:1",
  "3:2",
  "2:3",
  "16:9",
  "9:16",
] as const satisfies readonly ImageAspectRatio[];
const VIDEO_ASPECT_RATIOS = ["16:9", "9:16", "1:1"] as const satisfies readonly VideoAspectRatio[];

/**
 * Everything a generator can make. One entry per output: the Action it starts, the executors it
 * needs, and the options that output actually has. Adding an output is adding an entry.
 */
export const GENERATOR_KINDS = {
  image: {
    actionId: "generate-visualization",
    purpose: "image-generation",
    aspectRatios: IMAGE_ASPECT_RATIOS,
    durations: [],
    parameters: (title, { prompt, aspectRatio }) => ({ title, intent: prompt, aspectRatio }),
  },
  video: {
    actionId: "generate-video",
    purpose: "video-generation",
    aspectRatios: VIDEO_ASPECT_RATIOS,
    durations: VIDEO_DURATIONS,
    parameters: (title, { prompt, aspectRatio, durationSeconds }) => ({
      title,
      prompt,
      aspectRatio,
      durationSeconds,
    }),
  },
  document: {
    actionId: "draft-document",
    purpose: "knowledge",
    // A page: documents are read, not framed, so there is nothing to choose.
    aspectRatios: ["3:4"],
    durations: [],
    parameters: (title, { prompt }) => ({ title, purpose: prompt }),
  },
} as const satisfies Record<string, GeneratorKindDefinition>;

export type GeneratorKind = keyof typeof GENERATOR_KINDS;
export const GENERATOR_KIND_IDS = Object.keys(GENERATOR_KINDS) as GeneratorKind[];

/** The long edge of a generator's frame, in canvas units: large enough to read, small enough to place. */
const FRAME_LONG_EDGE = 360;

export function generatorFrameSize(aspectRatio: string) {
  const [width = 1, height = 1] = aspectRatio.split(":").map(Number);
  const scale = FRAME_LONG_EDGE / Math.max(width, height);
  return { width: width * scale, height: height * scale };
}

/** A result is named for what was asked: the prompt's first line, as long as a title reads. */
export function generatorTitle(prompt: string) {
  const [line = ""] = prompt.trim().split("\n");
  const title = line.trim();
  return title.length > 60 ? `${title.slice(0, 59).trimEnd()}…` : title;
}
