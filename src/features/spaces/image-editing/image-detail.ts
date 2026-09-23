export function applyImageDetail(
  source: Uint8ClampedArray,
  width: number,
  height: number,
  sharpness: number,
  clarity: number,
  grain: number,
): Uint8ClampedArray {
  if (sharpness === 0 && clarity === 0 && grain === 0) return source;
  const output = new Uint8ClampedArray(source);
  const strength = (sharpness / 100) * 1.2;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const offset = (y * width + x) * 4;
      if (source[offset + 3] === 0) continue;
      const left = x > 0 && source[offset - 1] !== 0 ? offset - 4 : -1;
      const right = x + 1 < width && source[offset + 7] !== 0 ? offset + 4 : -1;
      const top = y > 0 && source[offset - width * 4 + 3] !== 0 ? offset - width * 4 : -1;
      const bottom = y + 1 < height && source[offset + width * 4 + 3] !== 0 ? offset + width * 4 : -1;
      const count = Number(left >= 0) + Number(right >= 0) + Number(top >= 0) + Number(bottom >= 0);
      const centerLuma = 0.2126 * source[offset]! + 0.7152 * source[offset + 1]! + 0.0722 * source[offset + 2]!;
      const neighborLuma = count ? (
        (left >= 0 ? luminance(source, left) : 0) + (right >= 0 ? luminance(source, right) : 0)
        + (top >= 0 ? luminance(source, top) : 0) + (bottom >= 0 ? luminance(source, bottom) : 0)
      ) / count : centerLuma;
      const midWeight = 1 - Math.abs(centerLuma / 127.5 - 1);
      const clarityShift = (centerLuma - neighborLuma) * (clarity / 100) * 0.8 * midWeight;
      const hash = (Math.imul(x, 73856093) ^ Math.imul(y, 19349663)) & 255;
      const grainShift = (hash / 255 - 0.5) * (grain / 100) * 38;
      for (let channel = 0; channel < 3; channel++) {
        const center = source[offset + channel]!;
        const sum = (left >= 0 ? source[left + channel]! : 0)
          + (right >= 0 ? source[right + channel]! : 0)
          + (top >= 0 ? source[top + channel]! : 0)
          + (bottom >= 0 ? source[bottom + channel]! : 0);
        const mean = count ? sum / count : center;
        output[offset + channel] = center + strength * (center - mean) + clarityShift + grainShift;
      }
    }
  }
  return output;
}

function luminance(source: Uint8ClampedArray, offset: number) {
  return 0.2126 * source[offset]! + 0.7152 * source[offset + 1]! + 0.0722 * source[offset + 2]!;
}
