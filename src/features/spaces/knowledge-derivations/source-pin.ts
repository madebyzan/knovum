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

export function pinSource(spaceId: string, node: SpaceCanvasNode | undefined): SourcePin {
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

interface SourcePinRelease {
  /** The pin the session holds, absent when no session is open. */
  pinned: () => SourcePin | undefined;
  /** The source as it stands now. */
  live: () => SourcePin;
  release: () => void;
  /** State that, once it changes, may let a release refused before succeed. */
  retryWhen?: WatchSource[];
}

/** Calls `release` whenever the live source no longer matches the pin a session holds. */
export function releaseWhenSourceChanges({ pinned, live, release, retryWhen = [] }: SourcePinRelease) {
  watch([live, ...retryWhen], ([current]) => {
    const pin = pinned();
    if (pin && !samePin(pin, current as SourcePin)) release();
  });
}
