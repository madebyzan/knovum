const RESIZE_HANDLE_DIAMETER = 9;
const RESIZE_HANDLE_HIT_WIDTH = 14;

export function canvasChromeMetrics(zoom: number) {
  const screenPixelsToCanvasUnits = (pixels: number) => pixels / zoom;
  const resizeVisualScale = Math.sqrt(Math.min(zoom, 1));
  const resizeVisual = (pixels: number) => screenPixelsToCanvasUnits(pixels * resizeVisualScale);
  return {
    connector: {
      arrowHalfLength: screenPixelsToCanvasUnits(3.5),
      hitExtension: screenPixelsToCanvasUnits(13),
      offset: screenPixelsToCanvasUnits(20),
      radius: screenPixelsToCanvasUnits(9),
      strokeWidth: screenPixelsToCanvasUnits(1),
    },
    resize: {
      cornerRadius: resizeVisual(RESIZE_HANDLE_DIAMETER / 2),
      hitStrokeWidth: screenPixelsToCanvasUnits(RESIZE_HANDLE_HIT_WIDTH),
      size: resizeVisual(RESIZE_HANDLE_DIAMETER),
      strokeWidth: screenPixelsToCanvasUnits(1),
    },
  } as const;
}
