import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

const usagePurposeSchema = z.enum([
  "assistant",
  "editor",
  "knowledge",
  "knowledge_indexing",
  "translation",
  "visual_analysis",
]);

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

const tokenUsageReportSchema = z
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

export type TokenUsageReport = z.infer<typeof tokenUsageReportSchema>;
export type UsageDay = z.infer<typeof usageDaySchema>;
export type UsagePurpose = z.infer<typeof usagePurposeSchema>;

export async function loadTokenUsageReport(fromDate: string, toDate: string) {
  const value = await invoke<unknown>("plugin:ai|get_token_usage_report", {
    request: { fromDate, toDate },
  });
  return tokenUsageReportSchema.parse(value);
}
