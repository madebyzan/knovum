export interface NudgeCommandInput {
  id: string;
  titleKey: string;
  shortcut: string;
  x: number;
  y: number;
}

export const NUDGE_COMMANDS: readonly NudgeCommandInput[] = [
  { id: "spaces.nudge-left", titleKey: "spaces.canvas.nudgeLeft", shortcut: "ArrowLeft", x: -1, y: 0 },
  { id: "spaces.nudge-right", titleKey: "spaces.canvas.nudgeRight", shortcut: "ArrowRight", x: 1, y: 0 },
  { id: "spaces.nudge-up", titleKey: "spaces.canvas.nudgeUp", shortcut: "ArrowUp", x: 0, y: -1 },
  { id: "spaces.nudge-down", titleKey: "spaces.canvas.nudgeDown", shortcut: "ArrowDown", x: 0, y: 1 },
  {
    id: "spaces.nudge-left-large",
    titleKey: "spaces.canvas.nudgeLeftLarge",
    shortcut: "Shift+ArrowLeft",
    x: -10,
    y: 0,
  },
  {
    id: "spaces.nudge-right-large",
    titleKey: "spaces.canvas.nudgeRightLarge",
    shortcut: "Shift+ArrowRight",
    x: 10,
    y: 0,
  },
  {
    id: "spaces.nudge-up-large",
    titleKey: "spaces.canvas.nudgeUpLarge",
    shortcut: "Shift+ArrowUp",
    x: 0,
    y: -10,
  },
  {
    id: "spaces.nudge-down-large",
    titleKey: "spaces.canvas.nudgeDownLarge",
    shortcut: "Shift+ArrowDown",
    x: 0,
    y: 10,
  },
];
