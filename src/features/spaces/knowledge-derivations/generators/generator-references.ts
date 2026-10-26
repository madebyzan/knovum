import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { creationSources, type CreationSource } from "../catalog/creation-source";

/**
 * What a generator can make of a referenced node right now. A reference is held by node id and
 * pinned only when the person asks: the revision sent is the one they were looking at when they
 * pressed Generate, and a file that is still arriving is already a reference, just not yet a
 * readable one.
 */
export type ReferenceReading =
  | { state: "ready"; source: CreationSource }
  | { state: "preparing" }
  | { state: "unreadable" };

/**
 * `working` is whether the node has work queued or running on it (import, analysis, indexing):
 * such a node is about to become readable, so it waits instead of being refused.
 */
export function readReference(node: SpaceCanvasNode, working: boolean): ReferenceReading {
  const [source] = creationSources([node]);
  if (source) return { state: "ready", source };
  return working || arriving(node) ? { state: "preparing" } : { state: "unreadable" };
}

/** A resource whose file is still being imported or processed; a failed one will not arrive. */
function arriving({ data }: SpaceCanvasNode) {
  return (
    "processingState" in data &&
    (data.processingState === "importing" || data.processingState === "processing")
  );
}

/** Every reference pinned, or nothing: a generation never runs on part of what was asked. */
export function pinnedReferences(readings: readonly ReferenceReading[]): CreationSource[] | undefined {
  const sources = readings.flatMap((reading) => (reading.state === "ready" ? [reading.source] : []));
  return sources.length === readings.length ? sources : undefined;
}
