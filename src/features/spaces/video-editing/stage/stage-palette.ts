/** Design tokens the Konva stage needs as plain colours; Konva cannot resolve CSS variables. */
export interface StagePalette {
  accent: string;
  anchor: string;
}

export function readStagePalette(): StagePalette {
  const styles = getComputedStyle(document.documentElement);
  return {
    accent: token(styles, "--color-stroke-accent"),
    anchor: token(styles, "--color-surface-panel"),
  };
}

function token(styles: CSSStyleDeclaration, name: string) {
  const resolved = styles.getPropertyValue(name).trim();
  if (!resolved) throw new Error(`Video stage colour token ${name} is not defined`);
  return resolved;
}
