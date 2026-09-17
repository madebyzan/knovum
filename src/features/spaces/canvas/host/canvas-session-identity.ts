import type { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";

interface CanvasSessionIdentity {
  generation: number;
  queue: CanvasSnapshotSaveQueue | undefined;
  spaceId: string;
}

export function requireCurrentCanvasSession(expected: CanvasSessionIdentity, current: CanvasSessionIdentity) {
  if (
    current.queue !== expected.queue ||
    current.generation !== expected.generation ||
    current.spaceId !== expected.spaceId
  ) {
    throw new Error("Canvas transaction target changed");
  }
}

export function isCurrentCanvasSession(expected: CanvasSessionIdentity, current: CanvasSessionIdentity) {
  return (
    current.queue === expected.queue &&
    current.generation === expected.generation &&
    current.spaceId === expected.spaceId
  );
}
