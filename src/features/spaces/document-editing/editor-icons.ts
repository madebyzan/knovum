import { defineComponent, h, type VNode } from "vue";

type SvgSpec = readonly (readonly [tag: string, attrs: Record<string, string | number>])[];

export function createEditorSvgIcon(name: string, elements: SvgSpec) {
  return defineComponent({
    name,
    props: {
      strokeWidth: { type: [Number, String], default: 1.65 },
    },
    setup(props, { attrs }) {
      return (): VNode =>
        h(
          "svg",
          {
            viewBox: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            "stroke-width": props.strokeWidth,
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "aria-hidden": "true",
            ...attrs,
          },
          elements.map(([tag, elAttrs]) => h(tag, elAttrs)),
        );
    },
  });
}

/** Crisp radical-x (`√x`) glyph for inline KaTeX math. */
export const InlineMathIcon = createEditorSvgIcon("InlineMathIcon", [
  ["path", { d: "M3 14.25L6.25 19L10.25 6.25H21" }],
  ["path", { d: "M13.5 11.25L18.75 17.5" }],
  ["path", { d: "M18.75 11.25L13.5 17.5" }],
]);

/** Framed centered Sigma (`[Σ]`) glyph distinguishing display math blocks from inline math. */
export const BlockMathIcon = createEditorSvgIcon("BlockMathIcon", [
  ["rect", { x: 3, y: 4, width: 18, height: 16, rx: 3, fill: "currentColor", "fill-opacity": 0.06 }],
  ["path", { d: "M15.5 8.5H8.5L12.75 12L8.5 15.5H15.5" }],
]);

/** Geometric flowchart glyph for Mermaid diagrams (source node branching cleanly to two targets). */
export const MermaidDiagramIcon = createEditorSvgIcon("MermaidDiagramIcon", [
  ["rect", { x: 2.5, y: 9, width: 6.5, height: 6, rx: 1.75, fill: "currentColor", "fill-opacity": 0.08 }],
  ["rect", { x: 15, y: 3.5, width: 6.5, height: 5.5, rx: 1.75 }],
  ["rect", { x: 15, y: 15, width: 6.5, height: 5.5, rx: 1.75 }],
  ["path", { d: "M9 12H12" }],
  ["path", { d: "M15 6.25H13.25C12.56 6.25 12 6.81 12 7.5V16.5C12 17.19 12.56 17.75 13.25 17.75H15" }],
]);

/** Callout card glyph with left accent bar and body lines. */
export const CalloutBlockIcon = createEditorSvgIcon("CalloutBlockIcon", [
  ["rect", { x: 3, y: 4, width: 18, height: 16, rx: 3, fill: "currentColor", "fill-opacity": 0.06 }],
  ["rect", { x: 6.25, y: 7.75, width: 1.75, height: 8.5, rx: 0.875, fill: "currentColor", stroke: "none" }],
  ["path", { d: "M11 9.75H17.25" }],
  ["path", { d: "M11 14.25H15.25" }],
]);

/** Section divider glyph: crisp horizontal rule framed by subtle prose hints. */
export const DividerBlockIcon = createEditorSvgIcon("DividerBlockIcon", [
  ["path", { d: "M3 12H21" }],
  ["path", { d: "M6 6.5H14.5", opacity: 0.45 }],
  ["path", { d: "M6 17.5H18", opacity: 0.45 }],
]);

/** Featherlight 2×3 solid-dot Notion-style drag grip. */
export const DragGripIcon = createEditorSvgIcon("DragGripIcon", [
  ["circle", { cx: 9.25, cy: 6.5, r: 1.35, fill: "currentColor", stroke: "none" }],
  ["circle", { cx: 14.75, cy: 6.5, r: 1.35, fill: "currentColor", stroke: "none" }],
  ["circle", { cx: 9.25, cy: 12, r: 1.35, fill: "currentColor", stroke: "none" }],
  ["circle", { cx: 14.75, cy: 12, r: 1.35, fill: "currentColor", stroke: "none" }],
  ["circle", { cx: 9.25, cy: 17.5, r: 1.35, fill: "currentColor", stroke: "none" }],
  ["circle", { cx: 14.75, cy: 17.5, r: 1.35, fill: "currentColor", stroke: "none" }],
]);

/** Horizontal page-width expansion glyph (`|<->|`) for full-width document layout. */
export const PageWidthIcon = createEditorSvgIcon("PageWidthIcon", [
  ["path", { d: "M3.5 5V19" }],
  ["path", { d: "M20.5 5V19" }],
  ["path", { d: "M7 12H17" }],
  ["path", { d: "M9.75 9.25L7 12L9.75 14.75" }],
  ["path", { d: "M14.25 9.25L17 12L14.25 14.75" }],
]);

/** Formatting toolbar pill bar containing typography and rule strokes. */
export const FormatToolbarIcon = createEditorSvgIcon("FormatToolbarIcon", [
  ["rect", { x: 2.5, y: 5.5, width: 19, height: 13, rx: 3 }],
  ["path", { d: "M6.5 14.5L8.75 9.25L11 14.5" }],
  ["path", { d: "M7.25 13H10.25" }],
  ["path", { d: "M13.5 9.5V14.5", opacity: 0.4 }],
  ["path", { d: "M16 10.25H18.5" }],
  ["path", { d: "M16 13.75H18" }],
]);

/** Editorial polish glyph: precision pen nib refining a prose line (zero sparkle cliché). */
export const PolishTextIcon = createEditorSvgIcon("PolishTextIcon", [
  ["path", { d: "M4 19.5H13.5" }],
  [
    "path",
    {
      d: "M13.25 15.75L19.5 9.5C20.2 8.8 20.2 7.65 19.5 6.95L17.05 4.5C16.35 3.8 15.2 3.8 14.5 4.5L8.25 10.75L7 16L13.25 15.75Z",
    },
  ],
  ["path", { d: "M12.75 6.25L17.75 11.25" }],
]);

/** Prose continuation glyph: flowing lines extending into a forward continuation cursor. */
export const ContinueTextIcon = createEditorSvgIcon("ContinueTextIcon", [
  ["path", { d: "M4 6.5H19.5" }],
  ["path", { d: "M4 11.5H15.5" }],
  ["path", { d: "M4 16.5H11" }],
  ["path", { d: "M14.75 14L17.5 16.5L14.75 19" }],
]);

/** Side-by-side split document glyph for comparing original vs proposed rewrite. */
export const CompareDiffIcon = createEditorSvgIcon("CompareDiffIcon", [
  ["rect", { x: 3, y: 4, width: 18, height: 16, rx: 2.5 }],
  ["path", { d: "M12 4V20" }],
  ["path", { d: "M6 9H9.5", opacity: 0.55 }],
  ["path", { d: "M6 13H9", opacity: 0.55 }],
  ["path", { d: "M14.5 9H18" }],
  ["path", { d: "M14.5 13H18" }],
  ["path", { d: "M14.5 16.5H17" }],
]);
