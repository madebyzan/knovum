import { openMermaidLightbox } from "./mermaid-lightbox";
import { mountMermaidSvg } from "./mermaid-renderer";
import { bindPointerDrag, createControlButton, setButtonLabel } from "./mermaid-stage-controls";
import {
  computeInlineMermaidScale,
  DEFAULT_MERMAID_LABELS,
  type DiagramSize,
  type MermaidViewerLabels,
} from "./mermaid-viewport";

export interface MermaidStageOptions {
  readonly compact?: boolean;
  /** Shows the floating top-right fullscreen pill (used on Reading/UiMarkdown surfaces without an editor header). */
  readonly showExpandButton?: boolean;
  readonly labels?: MermaidViewerLabels;
  readonly onEdit?: () => void;
}

export interface MermaidStageHandle {
  update(svgMarkup: string, dark: boolean): boolean;
  setLabels(labels: MermaidViewerLabels): void;
  openFullscreen(): void;
  destroy(): void;
}

const VIEWPORT_PADDING_X = 48;
const VIEWPORT_PADDING_Y = 40;
const DEFAULT_MAX_STAGE_HEIGHT = 544;

function measureViewportBounds(viewport: HTMLElement, host: HTMLElement): DiagramSize {
  const rawW = viewport.clientWidth || host.clientWidth || 680;
  const rawH = viewport.clientHeight || DEFAULT_MAX_STAGE_HEIGHT;
  return {
    width: Math.max(200, rawW - VIEWPORT_PADDING_X),
    height: Math.max(160, rawH - VIEWPORT_PADDING_Y),
  };
}

function createCompactStage(host: HTMLElement, initialLabels: MermaidViewerLabels): MermaidStageHandle {
  let labels = initialLabels;
  let currentSvg = "";
  let currentDark = false;
  return {
    update(svgMarkup, dark) {
      currentSvg = svgMarkup;
      currentDark = dark;
      if (!svgMarkup) {
        host.shadowRoot?.replaceChildren();
        return false;
      }
      return Boolean(mountMermaidSvg(host, svgMarkup, { fitContainer: true }));
    },
    setLabels(next) {
      labels = next;
    },
    openFullscreen() {
      if (currentSvg) openMermaidLightbox({ svg: currentSvg, dark: currentDark, labels });
    },
    destroy() {},
  };
}

/** Creates a clean Mermaid stage with readable scaling, smooth scroll/drag pan, and fullscreen lightbox. */
export function createMermaidStage(host: HTMLElement, options: MermaidStageOptions = {}): MermaidStageHandle {
  let labels = options.labels ?? DEFAULT_MERMAID_LABELS;
  if (options.compact) return createCompactStage(host, labels);

  let currentSvg = "";
  let currentDark = false;
  let diagram: DiagramSize | null = null;

  host.classList.add("ui-mermaid-stage");
  const viewport = document.createElement("div");
  viewport.className = "ui-mermaid-viewport";
  const canvas = document.createElement("div");
  canvas.className = "ui-mermaid-canvas";
  viewport.append(canvas);

  const openFullscreen = () => {
    if (currentSvg) openMermaidLightbox({ svg: currentSvg, dark: currentDark, labels });
  };

  let expandPill: HTMLDivElement | null = null;
  let expandBtn: HTMLButtonElement | null = null;
  if (options.showExpandButton !== false) {
    expandPill = document.createElement("div");
    expandPill.className = "ui-mermaid-expand-pill";
    expandBtn = createControlButton({
      icon: "maximize",
      label: labels.preview,
      testId: "mermaid-fullscreen",
      onClick: openFullscreen,
    });
    expandPill.append(expandBtn);
    host.replaceChildren(viewport, expandPill);
  } else {
    host.replaceChildren(viewport);
  }

  const applyLayout = (centerHorizontal = false) => {
    if (!diagram) return;
    const bounds = measureViewportBounds(viewport, host);
    const scale = computeInlineMermaidScale(diagram, bounds.width);
    const renderedW = Math.max(1, Math.round(diagram.width * scale));
    const renderedH = Math.max(1, Math.round(diagram.height * scale));
    canvas.style.width = `${renderedW}px`;
    canvas.style.height = `${renderedH}px`;
    const overflowsX = renderedW > bounds.width + 2;
    const overflowsY = renderedH > bounds.height + 2 || viewport.scrollHeight > viewport.clientHeight + 2;
    viewport.dataset.pannable = overflowsX || overflowsY ? "true" : "false";
    if (centerHorizontal && overflowsX) {
      viewport.scrollLeft = Math.max(0, Math.round((viewport.scrollWidth - viewport.clientWidth) / 2));
    }
  };

  const unbindDrag = bindPointerDrag(viewport, {
    canStart: () => viewport.dataset.pannable === "true",
    onDelta: (dx, dy) => {
      viewport.scrollLeft -= dx;
      viewport.scrollTop -= dy;
    },
  });

  const onDblClick = () => (options.onEdit ? options.onEdit() : openFullscreen());
  viewport.addEventListener("dblclick", onDblClick);

  const resizeObserver =
    typeof ResizeObserver === "undefined" ? null : new ResizeObserver(() => applyLayout());
  resizeObserver?.observe(viewport);

  return {
    update(svgMarkup, dark) {
      currentSvg = svgMarkup;
      currentDark = dark;
      if (!svgMarkup) {
        diagram = null;
        canvas.shadowRoot?.replaceChildren();
        if (expandPill) expandPill.hidden = true;
        return false;
      }
      const next = mountMermaidSvg(canvas, svgMarkup);
      if (!next) return false;
      const isNew = !diagram || diagram.width !== next.width || diagram.height !== next.height;
      diagram = next;
      if (expandPill) expandPill.hidden = false;
      applyLayout(isNew);
      return true;
    },
    setLabels(next) {
      labels = next;
      if (expandBtn) setButtonLabel(expandBtn, next.preview);
    },
    openFullscreen,
    destroy() {
      unbindDrag();
      viewport.removeEventListener("dblclick", onDblClick);
      resizeObserver?.disconnect();
    },
  };
}
