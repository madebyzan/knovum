import { describe, expect, it } from "vitest";

import type { AudiobookRoleDraft, ChapterDraft } from "../audiobook-draft";
import { nextAudiobookRoleKey, renamedAudiobookDraft } from "../audiobook-role-draft";

const roles: AudiobookRoleDraft[] = [
  { id: "narrator", key: "Narrator", voiceRevisionId: "voice-1" },
  { id: "guest", key: "Guest", voiceRevisionId: "voice-2" },
];
const chapters: ChapterDraft[] = [
  {
    key: "chapter-1",
    title: "Chapter 1",
    paragraphs: [
      { key: "paragraph-1", text: "Opening", speakerKey: "Narrator" },
      { key: "paragraph-2", text: "Reply", speakerKey: "Guest" },
    ],
  },
];

describe("audiobook role drafts", () => {
  it("selects the first unused suffix for a new role", () => {
    expect(
      nextAudiobookRoleKey(" Narrator ", [...roles, { ...roles[0]!, id: "two", key: "Narrator 2" }]),
    ).toBe("Narrator 3");
  });

  it("renames the role and every paragraph bound to it without mutating the source", () => {
    const renamed = renamedAudiobookDraft(roles, chapters, "narrator", "Host");

    expect(renamed?.roles[0]?.key).toBe("Host");
    expect(renamed?.chapters[0]?.paragraphs.map((paragraph) => paragraph.speakerKey)).toEqual([
      "Host",
      "Guest",
    ]);
    expect(roles[0]?.key).toBe("Narrator");
    expect(chapters[0]?.paragraphs[0]?.speakerKey).toBe("Narrator");
  });

  it("rejects a duplicate role key", () => {
    expect(renamedAudiobookDraft(roles, chapters, "narrator", "Guest")).toBeUndefined();
  });
});
