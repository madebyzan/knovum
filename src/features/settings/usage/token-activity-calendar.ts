import { getDayOfWeek, getLocalTimeZone, parseDate, type CalendarDate } from "@internationalized/date";

import type { UsageDay } from "../../../foundation/ai/token-usage";

export interface TokenActivityCell {
  date: string;
  tokens: number;
  requests: number;
  level: 0 | 1 | 2 | 3 | 4;
  inRange: boolean;
}

export interface TokenActivityWeek {
  key: string;
  month?: string;
  cells: TokenActivityCell[];
}

export function buildTokenActivityWeeks(
  activity: UsageDay[],
  fromDate: string,
  toDate: string,
  locale: string,
): TokenActivityWeek[] {
  const totals = new Map(activity.map((day) => [day.date, day]));
  const from = parseDate(fromDate);
  const to = parseDate(toDate);
  const start = from.subtract({ days: getDayOfWeek(from, "en-US") });
  const end = to.add({ days: 6 - getDayOfWeek(to, "en-US") });
  const activeTotals = activity.map((day) => day.totalTokens).filter((value) => value > 0);
  const peak = Math.max(...activeTotals, 0);
  const weeks: TokenActivityWeek[] = [];
  let cursor = start;
  while (cursor.compare(end) <= 0) {
    const cells: TokenActivityCell[] = [];
    for (let index = 0; index < 7; index += 1) {
      const date = cursor.toString();
      const day = totals.get(date);
      cells.push({
        date,
        tokens: day?.totalTokens ?? 0,
        requests: day?.requests ?? 0,
        level: activityLevel(day?.totalTokens ?? 0, peak),
        inRange: cursor.compare(from) >= 0 && cursor.compare(to) <= 0,
      });
      cursor = cursor.add({ days: 1 });
    }
    weeks.push({ key: cells[0]!.date, cells });
  }
  attachMonthLabels(weeks, locale);
  return weeks;
}

function activityLevel(value: number, peak: number): TokenActivityCell["level"] {
  if (value === 0 || peak === 0) return 0;
  const ratio = value / peak;
  if (ratio <= 0.25) return 1;
  if (ratio <= 0.5) return 2;
  if (ratio <= 0.75) return 3;
  return 4;
}

function attachMonthLabels(weeks: TokenActivityWeek[], locale: string) {
  const timeZone = getLocalTimeZone();
  const months = new Intl.DateTimeFormat(locale, { month: "short" });
  let previousMonth = -1;
  for (const week of weeks) {
    const representative = representativeDate(week.cells);
    if (!representative || representative.month === previousMonth) continue;
    previousMonth = representative.month;
    week.month = months.format(representative.toDate(timeZone));
  }
}

function representativeDate(cells: TokenActivityCell[]): CalendarDate | undefined {
  const inRange = cells.find((cell) => cell.inRange);
  return inRange ? parseDate(inRange.date) : undefined;
}
