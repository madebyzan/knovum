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
/** Below any difference a person could see, above floating-point drift. */
const EPSILON = 1e-6;

/**
 * Places a picture of `picture` size in `stage`. On each axis a picture smaller than the stage is
 * centred, and a larger one is kept covering it — it can be panned to either edge, never past.
 */
export class ComparisonFrame {
  readonly fitted: ComparisonView;

  constructor(
    readonly stage: ComparisonSize,
    readonly picture: ComparisonSize,
  ) {
    this.fitted = this.placed(
      Math.min(
        1,
        (stage.width - FIT_MARGIN * 2) / picture.width,
        (stage.height - FIT_MARGIN * 2) / picture.height,
      ),
      0,
      0,
    );
  }

  /** One picture pixel per stage pixel, centred: what an upscale actually added is visible here. */
  get actualSize(): ComparisonView {
    return this.placed(
      1,
      (this.stage.width - this.picture.width) / 2,
      (this.stage.height - this.picture.height) / 2,
    );
  }

  /** Scales by `factor` around `point`, keeping the picture pixel under it in place where the bounds allow. */
  zoomed(view: ComparisonView, point: { x: number; y: number }, factor: number): ComparisonView {
    const scale = clamp(view.scale * factor, this.fitted.scale, MAX_SCALE);
    const applied = scale / view.scale;
    return this.placed(scale, point.x - (point.x - view.x) * applied, point.y - (point.y - view.y) * applied);
  }

  /** The next power-of-two zoom in `direction` — 50%, 100%, 200% — around the stage's centre. */
  stepped(view: ComparisonView, direction: 1 | -1): ComparisonView {
    const level = Math.log2(view.scale);
    const next = 2 ** (direction > 0 ? Math.floor(level + EPSILON) + 1 : Math.ceil(level - EPSILON) - 1);
    return this.zoomed(view, { x: this.stage.width / 2, y: this.stage.height / 2 }, next / view.scale);
  }

  canZoomOut(view: ComparisonView) {
    return view.scale > this.fitted.scale + EPSILON;
  }

  canZoomIn(view: ComparisonView) {
    return view.scale < MAX_SCALE - EPSILON;
  }

  panned(view: ComparisonView, dx: number, dy: number): ComparisonView {
    return this.placed(view.scale, view.x + dx, view.y + dy);
  }

  /** The part of the stage the picture covers. */
  span(view: ComparisonView) {
    return {
      left: Math.max(view.x, 0),
      right: Math.min(view.x + this.picture.width * view.scale, this.stage.width),
      top: Math.max(view.y, 0),
      bottom: Math.min(view.y + this.picture.height * view.scale, this.stage.height),
    };
  }

  private placed(scale: number, x: number, y: number): ComparisonView {
    return {
      scale,
      x: bounded(x, this.picture.width * scale, this.stage.width),
      y: bounded(y, this.picture.height * scale, this.stage.height),
    };
  }
}

function bounded(position: number, size: number, stage: number) {
  return size <= stage ? (stage - size) / 2 : clamp(position, stage - size, 0);
}

/** Whether two views show the picture the same way, as far as anyone could see. */
export function sameView(a: ComparisonView, b: ComparisonView) {
  return Math.abs(a.scale - b.scale) < EPSILON && Math.abs(a.x - b.x) < 0.5 && Math.abs(a.y - b.y) < 0.5;
}
