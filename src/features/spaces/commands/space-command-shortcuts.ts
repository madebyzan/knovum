export const SPACE_COMMAND_SHORTCUTS = {
  ask: "$mod+Enter",
  cancelInteraction: "Escape",
  lockSelection: "$mod+Shift+L",
  nextScene: "Alt+]",
  previousScene: "Alt+[",
  resetZoom: "$mod+0",
  zoomIn: "$mod+=",
  zoomOut: "$mod+-",
  zoomToFit: "Shift+2",
  zoomToSelection: "Shift+1",
} as const;

export const SPACE_COMMAND_KEYBINDINGS = {
  zoomIn: ["$mod+Equal", "$mod+Shift+Equal"],
  zoomToFit: ["Shift+Digit2"],
  zoomToSelection: ["Shift+Digit1"],
} as const;
