import { mountMermaidSvg } from "./mermaid-renderer";
import { bindPointerDrag, createControlButton } from "./mermaid-stage-controls";
import {
  clampMermaidZoom,
  computeLightboxFitScale,
  DEFAULT_MERMAID_LABELS,
  MERMAID_ZOOM_STEP,
  zoomAroundAnchor,
  type DiagramSize,
  type MermaidPanZoomState,
  type MermaidViewerLabels,
} from "./mermaid-viewport";

export interface MermaidLightboxOptions {
  readonly svg: string;
  readonly dark: boolean;
  readonly labels?: MermaidViewerLabels;
}

const PAN_KEY_STEP = 80;
let activeClose: (() => void) | null = null;

function createLightboxDom(dark: boolean) {
  const overlay = document.createElement("div");
  overlay.className = "ui-mermaid-lightbox";
  overlay.dataset.testid = "mermaid-lightbox";
  overlay.dataset.surfaceTheme = dark ? "dark" : "light";
  overlay.setAttribute("role", "dialog");
  overlay.setAttribute("aria-modal", "true");
  overlay.tabIndex = -1;

  const stage = document.createElement("div");
  stage.className = "ui-mermaid-lightbox-stage";
  const card = document.createElement("div");
  card.className = "ui-mermaid-lightbox-card";
  const svgHost = document.createElement("div");
  svgHost.className = "ui-mermaid-lightbox-svg";
  card.append(svgHost);
  stage.append(card);
  return { overlay, stage, card, svgHost };
}

/**
 * Opens the shared fullscreen vector Mermaid lightbox matching `SpaceTiptapImageLightbox`
 * visuals and controls (`ZoomOut | % | ZoomIn | Close`, wheel zoom, drag pan, Esc).
 */
export function openMermaidLightbox(options: MermaidLightboxOptions): () => void {
  activeClose?.();
  if (typeof document === "undefined" || !options.svg.trim()) return () => {};
  const labels = options.labels ?? DEFAULT_MERMAID_LABELS;
  const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const { overlay, stage, card, svgHost } = createLightboxDom(options.dark);

  const diagram: DiagramSize = mountMermaidSvg(svgHost, options.svg) ?? { width: 640, height: 320 };
  const fitScale = () =>
    computeLightboxFitScale(diagram, { width: globalThis.innerWidth, height: globalThis.innerHeight });

  let state: MermaidPanZoomState = { scale: fitScale(), x: 0, y: 0 };
  const pctBtn = document.createElement("button");
  pctBtn.type = "button";
  pctBtn.className = "ui-mermaid-btn ui-mermaid-pct";
  pctBtn.dataset.testid = "mermaid-lightbox-reset";

  const paint = () => {
    svgHost.style.width = `${Math.round(diagram.width * state.scale)}px`;
    svgHost.style.height = `${Math.round(diagram.height * state.scale)}px`;
    card.style.transform = `translate3d(${Math.round(state.x)}px, ${Math.round(state.y)}px, 0)`;
    pctBtn.textContent = `${Math.round((state.scale / fitScale()) * 100)}%`;
  };

  const resetTo = (scale: number) => {
    state = { scale: clampMermaidZoom(scale), x: 0, y: 0 };
    paint();
  };
  const stepZoom = (delta: number) => {
    state = zoomAroundAnchor(state, state.scale + delta, { x: 0, y: 0 });
    paint();
  };
  const panBy = (dx: number, dy: number) => {
    state = { ...state, x: state.x + dx, y: state.y + dy };
    paint();
  };

  const onKeydown = (event: KeyboardEvent) => {
    const actions: Record<string, () => void> = {
      Escape: close,
      "+": () => stepZoom(MERMAID_ZOOM_STEP),
      "=": () => stepZoom(MERMAID_ZOOM_STEP),
      "-": () => stepZoom(-MERMAID_ZOOM_STEP),
      "0": () => resetTo(fitScale()),
      ArrowLeft: () => panBy(PAN_KEY_STEP, 0),
      ArrowRight: () => panBy(-PAN_KEY_STEP, 0),
      ArrowUp: () => panBy(0, PAN_KEY_STEP),
      ArrowDown: () => panBy(0, -PAN_KEY_STEP),
    };
    const run = actions[event.key];
    if (!run) return;
    event.preventDefault();
    event.stopPropagation();
    run();
  };

  const unbindDrag = bindPointerDrag(stage, {
    onDelta: panBy,
    onClickEmpty: (event) => {
      if (event.target === stage) close();
    },
  });

  const close = () => {
    if (activeClose === close) activeClose = null;
    unbindDrag();
    globalThis.removeEventListener("keydown", onKeydown, true);
    overlay.remove();
    previousFocus?.focus?.();
  };

  pctBtn.addEventListener("click", (event) => {
    event.stopPropagation();
    resetTo(fitScale());
  });
  const sep = document.createElement("span");
  sep.className = "ui-mermaid-sep";
  const bar = document.createElement("div");
  bar.className = "ui-mermaid-lightbox-bar";
  bar.append(
    createControlButton({
      icon: "zoomOut",
      label: labels.zoomOut,
      testId: "mermaid-lightbox-zoom-out",
      onClick: () => stepZoom(-MERMAID_ZOOM_STEP),
    }),
    pctBtn,
    createControlButton({
      icon: "zoomIn",
      label: labels.zoomIn,
      testId: "mermaid-lightbox-zoom-in",
      onClick: () => stepZoom(MERMAID_ZOOM_STEP),
    }),
    sep,
    createControlButton({
      icon: "close",
      label: labels.close,
      testId: "mermaid-lightbox-close",
      onClick: close,
    }),
  );
  overlay.append(stage, bar);

  stage.addEventListener("dblclick", () => {
    const fit = fitScale();
    resetTo(Math.abs(state.scale - fit) < 0.08 ? fit * 1.75 : fit);
  });
  overlay.addEventListener(
    "wheel",
    (event) => {
      if (!event.deltaY) return;
      event.preventDefault();
      const rect = stage.getBoundingClientRect();
      const factor = event.deltaY < 0 ? 1.15 : 1 / 1.15;
      state = zoomAroundAnchor(state, state.scale * factor, {
        x: event.clientX - (rect.left + rect.width / 2),
        y: event.clientY - (rect.top + rect.height / 2),
      });
      paint();
    },
    { passive: false },
  );

  globalThis.addEventListener("keydown", onKeydown, true);
  document.body.append(overlay);
  paint();
  overlay.focus();
  activeClose = close;
  return close;
}
