export function applyImageGlow(
  source: Uint8ClampedArray,
  input: { width: number; height: number; amount: number; visibleSize: { width: number; height: number } },
): Uint8ClampedArray {
  const { width, height, amount, visibleSize } = input;
  if (amount === 0) return source;
  const mask = document.createElement("canvas");
  mask.width = width;
  mask.height = height;
  const maskContext = mask.getContext("2d", { willReadFrequently: true })!;
  const highlights = new Uint8ClampedArray(source.length);
  for (let offset = 0; offset < source.length; offset += 4) {
    const luminance = 0.2126 * source[offset]! + 0.7152 * source[offset + 1]! + 0.0722 * source[offset + 2]!;
    const value = source[offset + 3] === 0 ? 0 : Math.max(0, luminance - 140);
    highlights[offset] = value;
    highlights[offset + 1] = value;
    highlights[offset + 2] = value;
    highlights[offset + 3] = 255;
  }
  maskContext.putImageData(new ImageData(highlights, width, height), 0, 0);
  const blurred = document.createElement("canvas");
  blurred.width = width;
  blurred.height = height;
  const blurContext = blurred.getContext("2d", { willReadFrequently: true })!;
  blurContext.filter = `blur(${Math.max(1, Math.max(visibleSize.width, visibleSize.height) / 300)}px)`;
  blurContext.drawImage(mask, 0, 0);
  const bloom = blurContext.getImageData(0, 0, width, height).data;
  const output = new Uint8ClampedArray(source);
  for (let offset = 0; offset < output.length; offset += 4) {
    if (output[offset + 3] === 0) continue;
    const lift = (((bloom[offset]! / 255) * amount) / 100) * 0.8;
    for (let channel = 0; channel < 3; channel++) {
      const value = source[offset + channel]!;
      output[offset + channel] = value + (255 - value) * lift;
    }
  }
  return output;
}
