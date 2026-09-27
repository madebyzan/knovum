export const CANVAS_FONT_SIZES = [10, 12, 14, 18, 24, 36, 48, 64, 72, 96, 144] as const;

export const CANVAS_INLINE_TEXT_COLORS = [
  ["default", "var(--color-fg-primary)", "default"],
  ["var(--color-content-red)", "var(--color-content-red)", "red"],
  ["var(--color-content-amber)", "var(--color-content-amber)", "amber"],
  ["var(--color-content-yellow)", "var(--color-content-yellow)", "yellow"],
  ["var(--color-content-green)", "var(--color-content-green)", "green"],
  ["var(--color-content-teal)", "var(--color-content-teal)", "teal"],
  ["var(--color-content-cyan)", "var(--color-content-cyan)", "cyan"],
  ["var(--color-content-blue)", "var(--color-content-blue)", "blue"],
  ["var(--color-content-purple)", "var(--color-content-purple)", "purple"],
  ["var(--color-content-pink)", "var(--color-content-pink)", "pink"],
  ["var(--color-content-gray)", "var(--color-content-gray)", "gray"],
  ["var(--color-white)", "var(--color-white)", "white"],
] as const;
