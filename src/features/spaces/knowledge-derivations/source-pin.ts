import { watch, type WatchSource } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

/**
 * The exact source a canvas session works on: the Space, the asset revision and the edit rendered
 * over it. A session pinned to a source ends when any of the three changes, because whatever it
 * drafted describes a picture that is no longer there.
 */
export interface SourcePin {
  spaceId: string;
  sourceRevision: string | undefined;
  sourceEditKey: string | undefined;
}

export function sourcePin(spaceId: string, node: SpaceCanvasNode | undefined): SourcePin {
  return {
    spaceId,
    sourceRevision: node?.data.sourceRevision,
    sourceEditKey: node?.data.kind === "image" ? JSON.stringify(node.data.imageEdit ?? null) : undefined,
  };
}

function samePin(left: SourcePin, right: SourcePin) {
  return (
    left.spaceId === right.spaceId &&
    left.sourceRevision === right.sourceRevision &&
    left.sourceEditKey === right.sourceEditKey
  );
}

/** Calls `release` whenever the live source no longer matches the pin a session holds. */
export function releaseWhenSourceChanges(
  pinned: () => SourcePin | undefined,
  live: () => SourcePin,
  release: () => void,
  also: WatchSource[] = [],
) {
  watch([live, ...also], ([current]) => {
    const pin = pinned();
    if (pin && !samePin(pin, current as SourcePin)) release();
  });
}
