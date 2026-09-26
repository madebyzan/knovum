export interface NudgeCommandInput {
  id: string;
  titleKey: string;
  shortcut: string;
  x: number;
  y: number;
}

export const NUDGE_COMMANDS: readonly NudgeCommandInput[] = [
  { id: "spaces.nudge-left", titleKey: "spaces.canvas.nudge.left", shortcut: "ArrowLeft", x: -1, y: 0 },
  { id: "spaces.nudge-right", titleKey: "spaces.canvas.nudge.right", shortcut: "ArrowRight", x: 1, y: 0 },
  { id: "spaces.nudge-up", titleKey: "spaces.canvas.nudge.up", shortcut: "ArrowUp", x: 0, y: -1 },
  { id: "spaces.nudge-down", titleKey: "spaces.canvas.nudge.down", shortcut: "ArrowDown", x: 0, y: 1 },
  {
    id: "spaces.nudge-left-large",
    titleKey: "spaces.canvas.nudge.leftLarge",
    shortcut: "Shift+ArrowLeft",
    x: -10,
    y: 0,
  },
  {
    id: "spaces.nudge-right-large",
    titleKey: "spaces.canvas.nudge.rightLarge",
    shortcut: "Shift+ArrowRight",
    x: 10,
    y: 0,
  },
  {
    id: "spaces.nudge-up-large",
    titleKey: "spaces.canvas.nudge.upLarge",
    shortcut: "Shift+ArrowUp",
    x: 0,
    y: -10,
  },
  {
    id: "spaces.nudge-down-large",
    titleKey: "spaces.canvas.nudge.downLarge",
    shortcut: "Shift+ArrowDown",
    x: 0,
    y: 10,
  },
];
