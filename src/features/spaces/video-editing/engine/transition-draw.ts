import type { VideoTransitionKind } from "../project";

interface FrameSize {
  width: number;
  height: number;
}

type Painter = (
  context: CanvasRenderingContext2D,
  frames: { from: CanvasImageSource; to: CanvasImageSource },
  progress: number,
  size: FrameSize,
) => void;

function wipe(axis: "x" | "y", towards: -1 | 1): Painter {
  return (context, frames, progress, size) => {
    context.drawImage(frames.from, 0, 0, size.width, size.height);
    const span = axis === "x" ? size.width : size.height;
    const revealed = span * progress;
    const offset = towards === 1 ? 0 : span - revealed;
    context.save();
    context.beginPath();
    if (axis === "x") context.rect(offset, 0, revealed, size.height);
    else context.rect(0, offset, size.width, revealed);
    context.clip();
    context.drawImage(frames.to, 0, 0, size.width, size.height);
    context.restore();
  };
}

function slide(axis: "x" | "y", towards: -1 | 1): Painter {
  return (context, frames, progress, size) => {
    const span = axis === "x" ? size.width : size.height;
    const travel = span * progress * towards;
    const entry = travel - span * towards;
    const place = (image: CanvasImageSource, shift: number) =>
      context.drawImage(image, axis === "x" ? shift : 0, axis === "y" ? shift : 0, size.width, size.height);
    place(frames.from, travel);
    place(frames.to, entry);
  };
}

function fadeThrough(color: string): Painter {
  return (context, frames, progress, size) => {
    context.fillStyle = color;
    context.fillRect(0, 0, size.width, size.height);
    const first = progress < 0.5;
    context.globalAlpha = first ? 1 - progress * 2 : progress * 2 - 1;
    context.drawImage(first ? frames.from : frames.to, 0, 0, size.width, size.height);
    context.globalAlpha = 1;
  };
}

const dissolve: Painter = (context, frames, progress, size) => {
  context.drawImage(frames.from, 0, 0, size.width, size.height);
  context.globalAlpha = progress;
  context.drawImage(frames.to, 0, 0, size.width, size.height);
  context.globalAlpha = 1;
};

/**
 * Canvas equivalents of the FFmpeg `xfade` transitions the export uses, so the stage shows the
 * frame that will be rendered rather than an approximation.
 */
const painters: Record<VideoTransitionKind, Painter> = {
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

export interface TransitionFrame {
  kind: VideoTransitionKind;
  frames: { from: CanvasImageSource; to: CanvasImageSource };
  progress: number;
  size: FrameSize;
}

export function drawTransition(context: CanvasRenderingContext2D, frame: TransitionFrame) {
  const progress = Math.min(1, Math.max(0, frame.progress));
  painters[frame.kind](context, frame.frames, progress, frame.size);
}
