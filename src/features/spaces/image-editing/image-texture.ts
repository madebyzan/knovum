interface ImageTextureSettings {
  width: number;
  height: number;
  sharpness: number;
  clarity: number;
  grain: number;
}

interface ImageTextureContext {
  source: Uint8ClampedArray;
  output: Uint8ClampedArray;
  neighbors: Int32Array;
  settings: ImageTextureSettings;
  strength: number;
}

export function applyImageTexture(
  source: Uint8ClampedArray,
  settings: ImageTextureSettings,
): Uint8ClampedArray {
  if (settings.sharpness === 0 && settings.clarity === 0 && settings.grain === 0) return source;
  const output = new Uint8ClampedArray(source);
  const context: ImageTextureContext = {
    source,
    output,
    neighbors: new Int32Array(4),
    settings,
    strength: (settings.sharpness / 100) * 1.2,
  };
  for (let y = 0; y < settings.height; y++) {
    for (let x = 0; x < settings.width; x++) applyPixelTexture(context, x, y);
  }
  return output;
}

function applyPixelTexture(context: ImageTextureContext, x: number, y: number) {
  const { source, output, neighbors, settings, strength } = context;
  const offset = (y * settings.width + x) * 4;
  if (source[offset + 3] === 0) return;
  neighbors[0] = neighborOffset(source, offset - 4, x > 0);
  neighbors[1] = neighborOffset(source, offset + 4, x + 1 < settings.width);
  neighbors[2] = neighborOffset(source, offset - settings.width * 4, y > 0);
  neighbors[3] = neighborOffset(source, offset + settings.width * 4, y + 1 < settings.height);
  let count = 0;
  for (const neighbor of neighbors) if (neighbor >= 0) count++;
  const centerLuma = luminance(source, offset);
  const neighborLuma = count
    ? (0.2126 * neighborSum(source, neighbors, 0) +
        0.7152 * neighborSum(source, neighbors, 1) +
        0.0722 * neighborSum(source, neighbors, 2)) /
      count
    : centerLuma;
  const midWeight = 1 - Math.abs(centerLuma / 127.5 - 1);
  const clarityShift = (centerLuma - neighborLuma) * (settings.clarity / 100) * 0.8 * midWeight;
  const hash = (Math.imul(x, 73856093) ^ Math.imul(y, 19349663)) & 255;
  const grainShift = (hash / 255 - 0.5) * (settings.grain / 100) * 38;
  for (let channel = 0; channel < 3; channel++) {
    const center = source[offset + channel]!;
    const mean = count ? neighborSum(source, neighbors, channel) / count : center;
    output[offset + channel] = center + strength * (center - mean) + clarityShift + grainShift;
  }
}

function neighborOffset(source: Uint8ClampedArray, offset: number, inBounds: boolean) {
  return inBounds && source[offset + 3] !== 0 ? offset : -1;
}

function neighborSum(source: Uint8ClampedArray, neighbors: Int32Array, channel: number) {
  let sum = 0;
  for (const offset of neighbors) if (offset >= 0) sum += source[offset + channel]!;
  return sum;
}

function luminance(source: Uint8ClampedArray, offset: number) {
  return 0.2126 * source[offset]! + 0.7152 * source[offset + 1]! + 0.0722 * source[offset + 2]!;
}
