export function effectiveCanvasFontSize(baseFontSize: number, typographyScale: number) {
  return Math.round(baseFontSize * typographyScale);
}

export function canvasBaseFontSize(effectiveFontSize: number, typographyScale: number) {
  return effectiveFontSize / typographyScale;
}
