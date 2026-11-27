import type mermaid from "mermaid";

import { parseSvgDimensions, type DiagramSize } from "./mermaid-viewport";

export type MermaidRenderOutcome =
  | { readonly ok: true; readonly svg: string }
  | { readonly ok: false; readonly error: string };

const MERMAID_FONT_FAMILY = "'PingFang SC', -apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif";
const SHADOW_STYLE_FIT = [
  ":host { display: flex; align-items: center; justify-content: center; width: 100%; }",
  "svg { display: block; max-width: 100%; height: auto; margin: 0 auto; }",
].join("\n");
const SHADOW_STYLE_SCALED = [
  ":host { display: block; width: 100%; height: 100%; }",
  "svg { display: block; width: 100%; height: 100%; max-width: none; }",
].join("\n");

let mermaidLoader: Promise<typeof mermaid> | null = null;
let configuredTheme: "dark" | "light" | null = null;
let renderCounter = 0;

export function loadMermaidEngine() {
  mermaidLoader ??= Promise.all([import("mermaid"), import("@mermaid-js/layout-elk")]).then(
    ([mermaidModule, elkModule]) => {
      const engine = mermaidModule.default;
      engine.registerLayoutLoaders(elkModule.default);
      return engine;
    },
  );
  return mermaidLoader;
}

export function resetMermaidTheme() {
  configuredTheme = null;
}

export function buildMermaidThemeVariables(dark: boolean) {
  if (dark) {
    return {
      darkMode: true,
      background: "transparent",
      primaryColor: "#242429",
      primaryTextColor: "#ececef",
      primaryBorderColor: "#3f3f46",
      lineColor: "#8b92a0",
      secondaryColor: "#1d1d21",
      tertiaryColor: "#1b1b1f",
      clusterBkg: "#1b1b1f",
      clusterBorder: "#3a3a42",
      edgeLabelBackground: "#18181b",
      fontFamily: MERMAID_FONT_FAMILY,
      fontSize: "13px",
    };
  }
  return {
    darkMode: false,
    background: "transparent",
    primaryColor: "#ffffff",
    primaryTextColor: "#18181b",
    primaryBorderColor: "#d4d4d8",
    lineColor: "#71717a",
    secondaryColor: "#f8fafc",
    tertiaryColor: "#f8f9fb",
    clusterBkg: "#f8f9fb",
    clusterBorder: "#d4d4d8",
    edgeLabelBackground: "#ffffff",
    fontFamily: MERMAID_FONT_FAMILY,
    fontSize: "13px",
  };
}

export function buildMermaidThemeCss(dark: boolean): string {
  const clusterLabel = dark ? "#a1a1aa" : "#52525b";
  return [
    ".cluster rect { rx: 10px; ry: 10px; stroke-dasharray: 4 3; }",
    `.cluster-label span, .cluster-label p { color: ${clusterLabel}; font-weight: 600; font-size: 12px; }`,
    ".node rect { rx: 8px; ry: 8px; }",
    ".flowchart-link, .edgePath .path { stroke-width: 1.4px; stroke-linecap: round; stroke-linejoin: round; }",
  ].join("\n");
}

/**
 * Mounts a rendered Mermaid SVG inside an open Shadow Root so host prose stylesheets
 * (line-height, paragraph margins, overflow-wrap, font scaling) can never bleed into
 * Mermaid's `<foreignObject>` labels or distort diagram geometry.
 */
export function mountMermaidSvg(
  host: HTMLElement,
  svgMarkup: string,
  options: { readonly fitContainer?: boolean } = {},
): DiagramSize | null {
  if (typeof DOMParser === "undefined") return null;
  const parsed = new DOMParser().parseFromString(svgMarkup, "image/svg+xml");
  const root = parsed.documentElement;
  if (root.nodeName.toLowerCase() !== "svg") return null;
  const dimensions = parseSvgDimensions(root);
  if (!options.fitContainer) {
    root.style.removeProperty("max-width");
    root.setAttribute("width", String(dimensions.width));
    root.setAttribute("height", String(dimensions.height));
  }
  const shadow = host.shadowRoot ?? host.attachShadow({ mode: "open" });
  const style = document.createElement("style");
  style.textContent = options.fitContainer ? SHADOW_STYLE_FIT : SHADOW_STYLE_SCALED;
  shadow.replaceChildren(style, document.importNode(root, true));
  return dimensions;
}

export async function renderMermaidDiagram(code: string, dark: boolean): Promise<MermaidRenderOutcome> {
  const source = code.trim();
  if (!source) return { ok: true, svg: "" };
  const themeKey = dark ? "dark" : "light";
  const id = `knovum-mermaid-${++renderCounter}`;
  try {
    const engine = await loadMermaidEngine();
    if (configuredTheme !== themeKey) {
      engine.initialize({
        startOnLoad: false,
        securityLevel: "strict",
        theme: "base",
        layout: "elk",
        elk: {
          mergeEdges: false,
          nodePlacementStrategy: "NETWORK_SIMPLEX",
        },
        themeVariables: buildMermaidThemeVariables(dark),
        themeCSS: buildMermaidThemeCss(dark),
        flowchart: {
          htmlLabels: true,
          padding: 16,
          subGraphTitleMargin: { top: 10, bottom: 10 },
          useMaxWidth: false,
        },
        suppressErrorRendering: true,
      });
      configuredTheme = themeKey;
    }
    const rendered = await engine.render(id, source);
    return { ok: true, svg: rendered.svg };
  } catch (caught) {
    if (typeof document !== "undefined") document.getElementById(`d${id}`)?.remove();
    const message = caught instanceof Error ? caught.message : String(caught);
    return { ok: false, error: message };
  }
}
