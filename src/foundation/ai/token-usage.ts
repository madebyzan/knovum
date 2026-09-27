import { z } from "zod";

import { usagePurposes } from "../../shared/contracts/generated/contract-values";
import type { UsageReport } from "../../shared/contracts/generated/UsageReport";
import { invokeCommand } from "../ipc/invoke-command";

const usagePurposeSchema = z.enum(usagePurposes);

const usageDaySchema = z
  .object({
    date: z.iso.date(),
    inputTokens: z.number().int().nonnegative(),
    outputTokens: z.number().int().nonnegative(),
    totalTokens: z.number().int().nonnegative(),
    requests: z.number().int().nonnegative(),
  })
  .strict();

const usageSummarySchema = z
  .object({
    inputTokens: z.number().int().nonnegative(),
    outputTokens: z.number().int().nonnegative(),
    totalTokens: z.number().int().nonnegative(),
    requests: z.number().int().nonnegative(),
    activeDays: z.number().int().nonnegative(),
    peakDailyTokens: z.number().int().nonnegative(),
    currentStreakDays: z.number().int().nonnegative(),
    longestStreakDays: z.number().int().nonnegative(),
  })
  .strict();

const tokenUsageReportSchema: z.ZodType<UsageReport> = z
  .object({
    fromDate: z.iso.date(),
    toDate: z.iso.date(),
    summary: usageSummarySchema,
    activity: z.array(usageDaySchema),
    byPurpose: z.array(
      z
        .object({
          purpose: usagePurposeSchema,
          totalTokens: z.number().int().nonnegative(),
          requests: z.number().int().nonnegative(),
        })
        .strict(),
    ),
  })
  .strict();

export type { UsageReport as TokenUsageReport } from "../../shared/contracts/generated/UsageReport";
export type { UsageDay } from "../../shared/contracts/generated/UsageDay";
export type { UsagePurpose } from "../../shared/contracts/generated/UsagePurpose";

export async function loadTokenUsageReport(fromDate: string, toDate: string) {
  const value = await invokeCommand<unknown>("plugin:ai|get_token_usage_report", {
    request: { fromDate, toDate },
  });
  return tokenUsageReportSchema.parse(value);
}
