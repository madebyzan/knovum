import type { CanvasTool } from "../canvas/contract/canvas-tool";

export const CANVAS_TOOL_COMMANDS = [
  { tool: "select", title: "spaces.canvas.tools.select", shortcut: "V" },
  { tool: "pan", title: "shell.panCanvas", shortcut: "H" },
  { tool: "note", title: "spaces.editor.newNote", shortcut: "N" },
  { tool: "text", title: "spaces.editor.newText", shortcut: "T" },
  { tool: "section", title: "spaces.canvas.tools.section", shortcut: "S" },
] as const satisfies readonly { tool: CanvasTool; title: string; shortcut: string }[];
