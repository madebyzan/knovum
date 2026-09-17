export function yieldToCanvas(): Promise<void> {
  return new Promise((resolve) => globalThis.requestAnimationFrame(() => resolve()));
}
