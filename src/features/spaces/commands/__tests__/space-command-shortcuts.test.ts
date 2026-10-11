import { describe, expect, it } from "vitest";

import { SPACE_COMMAND_SHORTCUTS } from "../space-command-shortcuts";

describe("Space command shortcuts", () => {
  it("assigns one chord to each canvas command", () => {
    const shortcuts = Object.values(SPACE_COMMAND_SHORTCUTS);
    expect(new Set(shortcuts).size).toBe(shortcuts.length);
  });
});
