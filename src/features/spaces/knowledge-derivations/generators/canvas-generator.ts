import type { TaskTarget } from "../../../../shared/contracts/generated/TaskTarget";
import type { CanvasPoint, CanvasRect } from "../../canvas/contracts/canvas-editor-port";
import type { OutputFrame } from "../../contracts/generated/OutputFrame";
import { generatorFrameSize, type GeneratorKind, type GeneratorRequest } from "./generator-kinds";

/**
 * A frame on the canvas where something is about to be made: what to make, from what, by whom.
 * It is a draft of the session, not a node — nothing is stored until the person asks for it, and
 * then the output is reserved in this frame and takes its place.
 */
export interface CanvasGenerator extends GeneratorRequest {
  id: string;
  spaceId: string;
  kind: GeneratorKind;
  x: number;
  y: number;
  /** Canvas nodes to make it from, by id; pinned to their revision only when it generates. */
  references: readonly string[];
  target: TaskTarget | undefined;
  /** The Run it asked for, until that Run's output takes its place; "starting" until accepted. */
  submission: "starting" | { runId: string } | undefined;
}

export type GeneratorPlacement = { center: CanvasPoint } | { beside: CanvasRect };

export type GeneratorChanges = Partial<
  Pick<CanvasGenerator, "prompt" | "aspectRatio" | "durationSeconds" | "target">
>;

/** Room between a generator and what stands beside it, in canvas units. */
const GAP = 48;

/** Where a frame of this size stands: centred on a point, or to the right of a selection. */
export function generatorOrigin(
  placement: GeneratorPlacement,
  { width, height }: { width: number; height: number },
) {
  return "center" in placement
    ? { x: placement.center.x - width / 2, y: placement.center.y - height / 2 }
    : { x: placement.beside.x + placement.beside.width + GAP, y: placement.beside.y };
}

/** Where a file uploaded for a generator lands: to its left, so the frame stays where it was. */
export function uploadPosition(generator: CanvasGenerator): CanvasPoint {
  const { x, y, width } = generatorFrame(generator);
  return { x: x - GAP - width, y };
}

export function generatorFrame({ x, y, aspectRatio }: CanvasGenerator): OutputFrame {
  return { x, y, ...generatorFrameSize(aspectRatio) };
}

/** Asked for something, by someone, and not already at work on it. */
export function canGenerate({ prompt, target, submission }: CanvasGenerator) {
  return Boolean(prompt.trim() && target) && submission === undefined;
}

/** Nothing asked yet: leaving it loses nothing, so it does not stay behind as an empty frame. */
export function pristine({ prompt, references, submission }: CanvasGenerator) {
  return !prompt.trim() && references.length === 0 && submission === undefined;
}
