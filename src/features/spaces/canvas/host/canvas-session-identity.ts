export interface CanvasSessionIdentity {
  generation: number;
  spaceId: string;
}

export function requireCurrentCanvasSession(expected: CanvasSessionIdentity, current: CanvasSessionIdentity) {
  if (!isCurrentCanvasSession(expected, current)) {
    throw new Error("Canvas transaction target changed");
  }
}

export function isCurrentCanvasSession(expected: CanvasSessionIdentity, current: CanvasSessionIdentity) {
  return current.generation === expected.generation && current.spaceId === expected.spaceId;
}
