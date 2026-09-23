import { describe, expect, it } from "vitest";

import { buildTokenActivityWeeks } from "../usage/token-activity-calendar";

describe("token activity calendar", () => {
  it("aligns the requested interval to full Sunday-first weeks", () => {
    const weeks = buildTokenActivityWeeks(
      [
        { date: "2026-09-21", inputTokens: 50, outputTokens: 50, totalTokens: 100, requests: 1 },
        { date: "2026-09-22", inputTokens: 100, outputTokens: 300, totalTokens: 400, requests: 2 },
      ],
      "2026-09-21",
      "2026-09-22",
      "en",
    );

    expect(weeks).toHaveLength(1);
    expect(weeks[0]!.cells.map((cell) => cell.date)).toEqual([
      "2026-09-20",
      "2026-09-21",
      "2026-09-22",
      "2026-09-23",
      "2026-09-24",
      "2026-09-25",
      "2026-09-26",
    ]);
    expect(weeks[0]!.cells[1]).toMatchObject({ inRange: true, level: 1, tokens: 100 });
    expect(weeks[0]!.cells[2]).toMatchObject({ inRange: true, level: 4, tokens: 400 });
  });
});
