/**
 * Shared heading-level data for the Tiptap heading menus (block controls +
 * text style menu). Kept in one place so the two toolbar components stay in
 * sync and so the locale keys are referenced literally — a template string
 * like `t(\`spaces.richtext.heading${level}\`)` can't be found by "find
 * references" or renamed by tooling, and silently relies on a manual
 * whitelist entry in `dynamic-keys.ts` to pass the dead-key check.
 */
export const HEADING_LEVELS = [1, 2, 3, 4, 5, 6] as const;

export type HeadingLevel = (typeof HEADING_LEVELS)[number];

export const HEADING_LABEL_KEYS: Record<HeadingLevel, string> = {
  1: "spaces.richtext.heading1",
  2: "spaces.richtext.heading2",
  3: "spaces.richtext.heading3",
  4: "spaces.richtext.heading4",
  5: "spaces.richtext.heading5",
  6: "spaces.richtext.heading6",
};
