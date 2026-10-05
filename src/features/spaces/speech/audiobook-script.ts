import { z } from "zod";

const paragraph = z
  .object({ role: z.string().trim().min(1).max(120), text: z.string().trim().min(1).max(100_000) })
  .strict();
const chapter = z
  .object({ title: z.string().trim().min(1).max(200), paragraphs: z.array(paragraph).min(1).max(10_000) })
  .strict();
export const audiobookScript = z
  .object({ schemaVersion: z.literal(1), chapters: z.array(chapter).min(1).max(10_000) })
  .strict()
  .superRefine((value, context) => {
    const paragraphs = value.chapters.reduce((total, item) => total + item.paragraphs.length, 0);
    const characters = value.chapters.reduce(
      (total, item) => total + item.paragraphs.reduce((sum, entry) => sum + [...entry.text].length, 0),
      0,
    );
    if (paragraphs > 10_000) context.addIssue({ code: "custom", message: "Too many paragraphs" });
    if (characters > 1_000_000) context.addIssue({ code: "custom", message: "Script is too large" });
    const roles = new Set(value.chapters.flatMap((item) => item.paragraphs.map((entry) => entry.role)));
    if (roles.size > 1_000) context.addIssue({ code: "custom", message: "Too many roles" });
  });

export type ImportedAudiobookScript = z.infer<typeof audiobookScript>;
