export interface SurfaceRect {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

interface PlacementInput {
  selection: SurfaceRect;
  obstacles: readonly SurfaceRect[];
  viewport: { width: number; height: number };
  insets: { top: number; right: number; bottom: number; left: number };
  surface: { width: number; height: number };
  preferredAxis: "horizontal" | "vertical";
  gap: number;
}

interface Placement {
  left: number;
  top: number;
  maxHeight: number;
}

interface Candidate extends Placement {
  axis: "horizontal" | "vertical";
  width: number;
  visibleHeight: number;
  alignment: number;
  clearance: number;
}

interface SafeArea {
  left: number;
  top: number;
  right: number;
  bottom: number;
  height: number;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), Math.max(minimum, maximum));
}

function overlapArea(candidate: Candidate, obstacles: readonly SurfaceRect[]): number {
  return obstacles.reduce((area, obstacle) => {
    const width = Math.max(
      0,
      Math.min(candidate.left + candidate.width, obstacle.right + 8) -
        Math.max(candidate.left, obstacle.left - 8),
    );
    const height = Math.max(
      0,
      Math.min(candidate.top + candidate.visibleHeight, obstacle.bottom + 8) -
        Math.max(candidate.top, obstacle.top - 8),
    );
    return area + width * height;
  }, 0);
}

function horizontalCandidates(input: PlacementInput, safe: SafeArea): Candidate[] {
  const { selection, surface, gap } = input;
  const visibleHeight = Math.min(surface.height, safe.height);
  const centerY = (selection.top + selection.bottom) / 2;
  const candidates: Candidate[] = [];

  for (const left of [selection.right + gap, selection.left - gap - surface.width]) {
    if (left < safe.left || left + surface.width > safe.right) continue;
    const clearance =
      left > selection.right ? safe.right - selection.right - gap : selection.left - gap - safe.left;
    for (const [alignment, desiredTop] of [
      centerY - visibleHeight / 2,
      selection.top,
      selection.bottom - visibleHeight,
    ].entries()) {
      candidates.push({
        axis: "horizontal",
        left,
        top: clamp(desiredTop, safe.top, safe.bottom - visibleHeight),
        maxHeight: safe.height,
        visibleHeight,
        width: surface.width,
        alignment,
        clearance,
      });
    }
  }
  return candidates;
}

function verticalCandidates(input: PlacementInput, safe: SafeArea): Candidate[] {
  const { selection, surface, gap } = input;
  const centerX = (selection.left + selection.right) / 2;
  const candidates: Candidate[] = [];
  for (const [above, space] of [
    [true, selection.top - gap - safe.top],
    [false, safe.bottom - selection.bottom - gap],
  ] as const) {
    if (space <= 0 || surface.width > safe.right - safe.left) continue;
    const height = Math.min(surface.height, space);
    for (const [alignment, desiredLeft] of [
      centerX - surface.width / 2,
      selection.left,
      selection.right - surface.width,
    ].entries()) {
      candidates.push({
        axis: "vertical",
        left: clamp(desiredLeft, safe.left, safe.right - surface.width),
        top: above ? selection.top - gap - height : selection.bottom + gap,
        maxHeight: space,
        visibleHeight: height,
        width: surface.width,
        alignment,
        clearance: space,
      });
    }
  }
  return candidates;
}

/** Keep editing controls attached to the selected object while preserving nearby canvas content. */
export function placeSelectionSurface(input: PlacementInput): Placement {
  const { selection, obstacles, surface, preferredAxis, insets } = input;
  const safe: SafeArea = {
    left: insets.left,
    top: insets.top,
    right: input.viewport.width - insets.right,
    bottom: input.viewport.height - insets.bottom,
    height: Math.max(1, input.viewport.height - insets.bottom - insets.top),
  };
  const candidates = [...horizontalCandidates(input, safe), ...verticalCandidates(input, safe)];
  const visibleObstacles = obstacles.filter(
    ({ left, top, right: obstacleRight, bottom: obstacleBottom }) =>
      obstacleRight >= safe.left && left <= safe.right && obstacleBottom >= safe.top && top <= safe.bottom,
  );
  const ranked = candidates.map((candidate) => ({
    candidate,
    overlap: overlapArea(candidate, visibleObstacles),
  }));
  ranked.sort((first, second) => {
    const overlap = first.overlap - second.overlap;
    if (overlap) return overlap;
    const firstCandidate = first.candidate;
    const secondCandidate = second.candidate;
    if (firstCandidate.axis !== secondCandidate.axis) return firstCandidate.axis === preferredAxis ? -1 : 1;
    if (firstCandidate.visibleHeight !== secondCandidate.visibleHeight)
      return secondCandidate.visibleHeight - firstCandidate.visibleHeight;
    if (firstCandidate.axis === "horizontal" && firstCandidate.clearance !== secondCandidate.clearance)
      return secondCandidate.clearance - firstCandidate.clearance;
    return firstCandidate.alignment - secondCandidate.alignment;
  });

  const chosen = ranked[0]?.candidate;
  if (chosen) return { left: chosen.left, top: chosen.top, maxHeight: chosen.maxHeight };
  return {
    left: clamp(
      (selection.left + selection.right - surface.width) / 2,
      safe.left,
      safe.right - surface.width,
    ),
    top: safe.top,
    maxHeight: safe.height,
  };
}
