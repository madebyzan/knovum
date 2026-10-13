import type { VideoOutput, VideoTransitionKind } from "../project";

interface ClipRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface LayerStyle {
  opacity: number;
  dx: number;
  dy: number;
  clip?: ClipRect | undefined;
}

export interface TransitionStyle {
  /** Colour the frame passes through; the layers fade over it. */
  backdrop?: string | undefined;
  from: LayerStyle;
  to: LayerStyle;
}

const PLAIN: LayerStyle = { opacity: 1, dx: 0, dy: 0 };

function wipe(axis: "x" | "y", towards: -1 | 1) {
  return (progress: number, output: VideoOutput): TransitionStyle => {
    const span = axis === "x" ? output.width : output.height;
    const revealed = span * progress;
    const offset = towards === 1 ? 0 : span - revealed;
    const clip =
      axis === "x"
        ? { x: offset, y: 0, width: revealed, height: output.height }
        : { x: 0, y: offset, width: output.width, height: revealed };
    return { from: PLAIN, to: { ...PLAIN, clip } };
  };
}

function slide(axis: "x" | "y", towards: -1 | 1) {
  return (progress: number, output: VideoOutput): TransitionStyle => {
    const span = axis === "x" ? output.width : output.height;
    const travel = span * progress * towards;
    const shift = (value: number) => ({
      ...PLAIN,
      dx: axis === "x" ? value : 0,
      dy: axis === "y" ? value : 0,
    });
    return { from: shift(travel), to: shift(travel - span * towards) };
  };
}

function fadeThrough(color: string) {
  return (progress: number): TransitionStyle => ({
    backdrop: color,
    from: { ...PLAIN, opacity: Math.max(0, 1 - progress * 2) },
    to: { ...PLAIN, opacity: Math.max(0, progress * 2 - 1) },
  });
}

function dissolve(progress: number): TransitionStyle {
  return { from: PLAIN, to: { ...PLAIN, opacity: progress } };
}

/**
 * Scene-graph equivalents of the FFmpeg `xfade` transitions the export uses: clipping for wipes,
 * displacement for slides and opacity for the dissolves, so the stage previews the rendered cut.
 */
const styles: Record<VideoTransitionKind, (progress: number, output: VideoOutput) => TransitionStyle> = {
  dissolve,
  fadeBlack: fadeThrough("#000000"),
  fadeWhite: fadeThrough("#FFFFFF"),
  wipeLeft: wipe("x", -1),
  wipeRight: wipe("x", 1),
  wipeUp: wipe("y", -1),
  wipeDown: wipe("y", 1),
  slideLeft: slide("x", -1),
  slideRight: slide("x", 1),
  slideUp: slide("y", -1),
  slideDown: slide("y", 1),
};

export function transitionStyle(
  transition: { kind: VideoTransitionKind; progress: number },
  output: VideoOutput,
): TransitionStyle {
  const progress = Math.min(1, Math.max(0, transition.progress));
  return styles[transition.kind](progress, output);
}

export const PLAIN_LAYER = PLAIN;
