interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function selectionRectIntersects(left: Rect, right: Rect) {
  return (
    left.x <= right.x + right.width &&
    left.x + left.width >= right.x &&
    left.y <= right.y + right.height &&
    left.y + left.height >= right.y
  );
}
