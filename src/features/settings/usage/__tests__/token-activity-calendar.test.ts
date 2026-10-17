import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import { i18n } from "../../../../foundation/i18n/instance";
import type { UsageReport } from "../../../../shared/contracts/generated/UsageReport";
import { buildTokenActivityWeeks } from "../token-activity-calendar";
import TokenActivityCalendar from "../TokenActivityCalendar.vue";

describe("token activity calendar", () => {
  it("aligns the requested interval to full Sunday-first weeks", () => {
    const weeks = buildTokenActivityWeeks({
      activity: [
        { date: "2026-09-21", inputTokens: 50, outputTokens: 50, totalTokens: 100, requests: 1 },
        { date: "2026-09-22", inputTokens: 100, outputTokens: 300, totalTokens: 400, requests: 2 },
      ],
      fromDate: "2026-09-21",
      toDate: "2026-09-22",
      locale: "en",
    });

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

  it("includes the date and token count in an active day's tooltip", () => {
    const report: UsageReport = {
      fromDate: "2026-09-22",
      toDate: "2026-09-22",
      activity: [{ date: "2026-09-22", inputTokens: 100, outputTokens: 300, totalTokens: 400, requests: 1 }],
      summary: {
        inputTokens: 100,
        outputTokens: 300,
        totalTokens: 400,
        requests: 1,
        activeDays: 1,
        peakDailyTokens: 400,
        currentStreakDays: 1,
        longestStreakDays: 1,
      },
      byPurpose: [],
    };
    const wrapper = mount(TokenActivityCalendar, {
      props: { report },
      global: {
        plugins: [i18n],
        stubs: { UiTooltip: { props: ["text"], template: '<div :data-text="text"><slot /></div>' } },
      },
    });

    expect(wrapper.get('[data-text*="400 Token"]').attributes("data-text")).toContain("Sep 22, 2026");
    wrapper.unmount();
  });
});
