const SVG_NS = "http://www.w3.org/2000/svg";

export type MermaidControlIcon = "zoomOut" | "zoomIn" | "maximize" | "close";

const ICON_PATHS: Record<MermaidControlIcon, readonly string[]> = {
  zoomOut: ["M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z", "m21 21-4.35-4.35", "M8 11h6"],
  zoomIn: ["M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z", "m21 21-4.35-4.35", "M11 8v6", "M8 11h6"],
  maximize: ["M15 3h6v6", "M9 21H3v-6", "M21 3l-7 7", "M3 21l7-7"],
  close: ["M18 6 6 18", "m6 6 12 12"],
};

function createControlSvg(icon: MermaidControlIcon, size = 14): SVGSVGElement {
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("width", String(size));
  svg.setAttribute("height", String(size));
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-width", "1.65");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  svg.setAttribute("aria-hidden", "true");
  for (const d of ICON_PATHS[icon]) {
    const path = document.createElementNS(SVG_NS, "path");
    path.setAttribute("d", d);
    svg.append(path);
  }
  return svg;
}

export function createControlButton(options: {
  readonly icon: MermaidControlIcon;
  readonly label: string;
  readonly testId?: string;
  readonly onClick: () => void;
}): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "ui-mermaid-btn";
  button.setAttribute("aria-label", options.label);
  button.title = options.label;
  if (options.testId) button.dataset.testid = options.testId;
  button.append(createControlSvg(options.icon));
  button.addEventListener("click", (event) => {
    event.stopPropagation();
    options.onClick();
  });
  return button;
}

export function setButtonLabel(button: HTMLButtonElement, label: string) {
  button.setAttribute("aria-label", label);
  button.title = label;
}

/** Binds left-click pointer drag deltas on a stage or viewport element. */
export function bindPointerDrag(
  target: HTMLElement,
  handlers: {
    readonly canStart?: () => boolean;
    readonly onDelta: (dx: number, dy: number) => void;
    readonly onClickEmpty?: (event: MouseEvent) => void;
  },
): () => void {
  let pointerId: number | null = null;
  let lastX = 0;
  let lastY = 0;
  let dragged = false;

  const stop = (event?: PointerEvent) => {
    if (event && pointerId !== null && event.pointerId !== pointerId) return;
    if (pointerId !== null && target.hasPointerCapture?.(pointerId)) {
      target.releasePointerCapture(pointerId);
    }
    pointerId = null;
    target.dataset.dragging = "false";
  };

  const onDown = (event: PointerEvent) => {
    if (event.button !== 0 || (handlers.canStart && !handlers.canStart())) return;
    event.preventDefault();
    pointerId = event.pointerId;
    lastX = event.clientX;
    lastY = event.clientY;
    dragged = false;
    target.dataset.dragging = "true";
    target.setPointerCapture?.(event.pointerId);
  };

  const onMove = (event: PointerEvent) => {
    if (pointerId === null || event.pointerId !== pointerId) return;
    const dx = event.clientX - lastX;
    const dy = event.clientY - lastY;
    if (Math.abs(dx) > 2 || Math.abs(dy) > 2) dragged = true;
    lastX = event.clientX;
    lastY = event.clientY;
    handlers.onDelta(dx, dy);
  };

  const onClick = (event: MouseEvent) => {
    if (!dragged) handlers.onClickEmpty?.(event);
  };

  target.addEventListener("pointerdown", onDown);
  target.addEventListener("pointermove", onMove);
  target.addEventListener("pointerup", stop);
  target.addEventListener("pointercancel", stop);
  if (handlers.onClickEmpty) target.addEventListener("click", onClick);

  return () => {
    stop();
    target.removeEventListener("pointerdown", onDown);
    target.removeEventListener("pointermove", onMove);
    target.removeEventListener("pointerup", stop);
    target.removeEventListener("pointercancel", stop);
    if (handlers.onClickEmpty) target.removeEventListener("click", onClick);
  };
}
