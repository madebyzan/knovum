import {
  buildMermaidThemeCss,
  buildMermaidThemeVariables,
  loadMermaidEngine,
  resetMermaidTheme,
} from "./mermaid-renderer";

const MAX_EXPORT_EDGE_PX = 2400;
const EXPORT_PADDING_PX = 24;
const EXPORT_SCALE = 2;

let exportCounter = 0;

interface SvgGeometry {
  readonly width: number;
  readonly height: number;
  readonly markup: string;
}

function parseSvgDimension(value: string | null): number {
  if (!value) return 0;
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0;
}

function readSvgSize(svg: Element): { readonly width: number; readonly height: number } {
  const parts = svg
    .getAttribute("viewBox")
    ?.trim()
    .split(/[\s,]+/u)
    .map(Number);
  if (parts?.length === 4 && parts.every(Number.isFinite) && parts[2]! > 0 && parts[3]! > 0) {
    return { width: Math.ceil(parts[2]!), height: Math.ceil(parts[3]!) };
  }
  return {
    width: Math.ceil(parseSvgDimension(svg.getAttribute("width")) || 800),
    height: Math.ceil(parseSvgDimension(svg.getAttribute("height")) || 400),
  };
}

function normalizeSvgForRaster(svgMarkup: string): SvgGeometry | null {
  if (typeof DOMParser === "undefined" || typeof XMLSerializer === "undefined") return null;
  const doc = new DOMParser().parseFromString(svgMarkup, "image/svg+xml");
  const svg = doc.documentElement;
  if (svg.nodeName.toLowerCase() !== "svg") return null;

  const { width, height } = readSvgSize(svg);
  svg.setAttribute("xmlns", "http://www.w3.org/2000/svg");
  svg.setAttribute("width", String(width));
  svg.setAttribute("height", String(height));
  svg.style.maxWidth = "none";

  return { width, height, markup: new XMLSerializer().serializeToString(svg) };
}

function loadSvgImage(svgMarkup: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.decoding = "sync";
    img.addEventListener("load", () => resolve(img), { once: true });
    img.addEventListener("error", () => reject(new Error("Failed to decode rendered Mermaid SVG")), {
      once: true,
    });
    const encoded = encodeURIComponent(svgMarkup);
    img.src = `data:image/svg+xml;charset=utf-8,${encoded}`;
  });
}

async function rasterizeSvgToPngBase64(geometry: SvgGeometry): Promise<string | null> {
  if (typeof document === "undefined") return null;
  const img = await loadSvgImage(geometry.markup);
  const rawW = geometry.width + EXPORT_PADDING_PX * 2;
  const rawH = geometry.height + EXPORT_PADDING_PX * 2;
  const maxDim = Math.max(rawW, rawH, 1);
  const scale = Math.min(EXPORT_SCALE, MAX_EXPORT_EDGE_PX / maxDim);
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(rawW * scale));
  canvas.height = Math.max(1, Math.round(rawH * scale));
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;

  ctx.fillStyle = "#ffffff";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.scale(scale, scale);
  ctx.drawImage(img, EXPORT_PADDING_PX, EXPORT_PADDING_PX, geometry.width, geometry.height);

  const dataUrl = canvas.toDataURL("image/png");
  const prefix = "data:image/png;base64,";
  return dataUrl.startsWith(prefix) ? dataUrl.slice(prefix.length) : null;
}

/**
 * Renders a Mermaid diagram using pure SVG text labels (`htmlLabels: false`) so rasterization
 * onto a 2x Retina `<canvas>` never taints WKWebView/Safari, returning base64 PNG bytes suitable
 * for embedding in HTML, DOCX, PDF, and PPTX exports.
 */
export async function renderMermaidToPngBase64(code: string): Promise<string | null> {
  const source = code.trim();
  if (!source || typeof document === "undefined") return null;
  const id = `knovum-mermaid-export-${++exportCounter}`;
  try {
    const engine = await loadMermaidEngine();
    engine.initialize({
      startOnLoad: false,
      securityLevel: "strict",
      theme: "base",
      layout: "elk",
      elk: { mergeEdges: false, nodePlacementStrategy: "NETWORK_SIMPLEX" },
      themeVariables: buildMermaidThemeVariables(false),
      themeCSS: buildMermaidThemeCss(false),
      flowchart: {
        htmlLabels: false,
        padding: 18,
        subGraphTitleMargin: { top: 10, bottom: 10 },
        useMaxWidth: false,
      },
      suppressErrorRendering: true,
    });
    const rendered = await engine.render(id, source);
    const geometry = normalizeSvgForRaster(rendered.svg);
    if (!geometry) return null;
    return await rasterizeSvgToPngBase64(geometry);
  } catch {
    return null;
  } finally {
    document.getElementById(`d${id}`)?.remove();
    resetMermaidTheme();
  }
}
