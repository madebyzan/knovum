import { createEditorSvgIcon } from "./editor-icons";

/** Cohesive table-row & table-column manipulation icon family on Lucide's 24px optical grid. */
export const TableRowAboveIcon = createEditorSvgIcon("TableRowAboveIcon", [
  ["rect", { x: 3, y: 13.25, width: 18, height: 6.25, rx: 1.75, fill: "currentColor", "fill-opacity": 0.08 }],
  ["path", { d: "M12 4.5V9.5" }],
  ["path", { d: "M9.5 7H14.5" }],
]);

export const TableRowBelowIcon = createEditorSvgIcon("TableRowBelowIcon", [
  ["rect", { x: 3, y: 4.5, width: 18, height: 6.25, rx: 1.75, fill: "currentColor", "fill-opacity": 0.08 }],
  ["path", { d: "M12 14.5V19.5" }],
  ["path", { d: "M9.5 17H14.5" }],
]);

export const TableRowDeleteIcon = createEditorSvgIcon("TableRowDeleteIcon", [
  ["rect", { x: 3, y: 4.5, width: 18, height: 6.25, rx: 1.75, fill: "currentColor", "fill-opacity": 0.08 }],
  ["path", { d: "M9.75 14.75L14.25 19.25" }],
  ["path", { d: "M14.25 14.75L9.75 19.25" }],
]);

export const TableColLeftIcon = createEditorSvgIcon("TableColLeftIcon", [
  ["rect", { x: 13.25, y: 3, width: 6.25, height: 18, rx: 1.75, fill: "currentColor", "fill-opacity": 0.08 }],
  ["path", { d: "M7 9.5V14.5" }],
  ["path", { d: "M4.5 12H9.5" }],
]);

export const TableColRightIcon = createEditorSvgIcon("TableColRightIcon", [
  ["rect", { x: 4.5, y: 3, width: 6.25, height: 18, rx: 1.75, fill: "currentColor", "fill-opacity": 0.08 }],
  ["path", { d: "M17 9.5V14.5" }],
  ["path", { d: "M14.5 12H19.5" }],
]);

export const TableColDeleteIcon = createEditorSvgIcon("TableColDeleteIcon", [
  ["rect", { x: 4.5, y: 3, width: 6.25, height: 18, rx: 1.75, fill: "currentColor", "fill-opacity": 0.08 }],
  ["path", { d: "M14.75 9.75L19.25 14.25" }],
  ["path", { d: "M19.25 9.75L14.75 14.25" }],
]);

export const TableHeaderRowIcon = createEditorSvgIcon("TableHeaderRowIcon", [
  ["rect", { x: 3, y: 4, width: 18, height: 16, rx: 2.5 }],
  ["path", { d: "M3 9.5H21" }],
  [
    "rect",
    {
      x: 3.75,
      y: 4.75,
      width: 16.5,
      height: 4.25,
      rx: 1.25,
      fill: "currentColor",
      "fill-opacity": 0.22,
      stroke: "none",
    },
  ],
  ["path", { d: "M9.5 9.5V20", opacity: 0.55 }],
  ["path", { d: "M15 9.5V20", opacity: 0.55 }],
]);
