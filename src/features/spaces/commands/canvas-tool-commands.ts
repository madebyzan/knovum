import type { CanvasTool } from "../canvas/contracts/canvas-tool";

export const CANVAS_TOOL_COMMANDS = [
  { tool: "select", title: "spaces.canvas.tools.select", shortcut: "V" },
  { tool: "pan", title: "spaces.canvas.shortcuts.panCanvas", shortcut: "H" },
  { tool: "note", title: "spaces.editor.note.new", shortcut: "N" },
  { tool: "text", title: "spaces.editor.text.new", shortcut: "T" },
  { tool: "section", title: "spaces.canvas.tools.section", shortcut: "S" },
] as const satisfies readonly { tool: CanvasTool; title: string; shortcut: string }[];
