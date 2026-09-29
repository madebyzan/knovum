interface ImageRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export function drawRemovalMask(
  context: CanvasRenderingContext2D,
  rect: ImageRect,
  pixelRatio: number,
  strokes: readonly { readonly diameter: number; readonly points: readonly number[] }[],
) {
  context.save();
  context.scale(pixelRatio, pixelRatio);
  context.lineCap = "round";
  context.lineJoin = "round";
  context.strokeStyle = "rgba(45, 111, 239, 0.58)";
  context.fillStyle = context.strokeStyle;
  for (const stroke of strokes) {
    const points = stroke.points;
    const width = stroke.diameter * Math.min(rect.width, rect.height);
    context.lineWidth = width;
    const x = rect.left + points[0]! * rect.width;
    const y = rect.top + points[1]! * rect.height;
    if (points.length === 2 || (points.length === 4 && points[0] === points[2] && points[1] === points[3])) {
      context.beginPath();
      context.arc(x, y, width / 2, 0, Math.PI * 2);
      context.fill();
      continue;
    }
    context.beginPath();
    context.moveTo(x, y);
    for (let index = 2; index < points.length; index += 2) {
      context.lineTo(rect.left + points[index]! * rect.width, rect.top + points[index + 1]! * rect.height);
    }
    context.stroke();
  }
  context.restore();
}
