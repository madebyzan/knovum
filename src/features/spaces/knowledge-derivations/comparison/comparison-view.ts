import { clamp } from "es-toolkit";

export interface ComparisonSize {
  width: number;
  height: number;
}

/** Where the picture sits in the stage: its top-left corner and how many stage pixels one picture pixel spans. */
export interface ComparisonView {
  scale: number;
  x: number;
  y: number;
}

/** Room kept around a fitted picture, so its edges read against the backdrop. */
const FIT_MARGIN = 48;
/** Eight stage pixels per picture pixel is past the point where an upscale can show any more detail. */
const MAX_SCALE = 8;

/** The whole picture, centred — never enlarged past its own resolution, which would only show blur. */
export function fittedView(stage: ComparisonSize, picture: ComparisonSize): ComparisonView {
  const scale = Math.min(
    1,
    (stage.width - FIT_MARGIN * 2) / picture.width,
    (stage.height - FIT_MARGIN * 2) / picture.height,
  );
  return centred(stage, picture, scale);
}

/** One picture pixel per stage pixel, centred. */
export function actualSizeView(stage: ComparisonSize, picture: ComparisonSize): ComparisonView {
  return centred(stage, picture, 1);
}

/** Scales the view by `factor` around `point`, keeping the picture pixel under it in place, between the fitted view and `MAX_SCALE`. */
export function zoomedView(
  view: ComparisonView,
  point: { x: number; y: number },
  factor: number,
  fitted: ComparisonView,
): ComparisonView {
  const scale = clamp(view.scale * factor, fitted.scale, MAX_SCALE);
  const applied = scale / view.scale;
  return { scale, x: point.x - (point.x - view.x) * applied, y: point.y - (point.y - view.y) * applied };
}

function centred(stage: ComparisonSize, picture: ComparisonSize, scale: number): ComparisonView {
  return {
    scale,
    x: (stage.width - picture.width * scale) / 2,
    y: (stage.height - picture.height * scale) / 2,
  };
}
