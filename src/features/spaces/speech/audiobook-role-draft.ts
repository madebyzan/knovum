import type { AudiobookRoleDraft, ChapterDraft } from "./audiobook-draft";

export function nextAudiobookRoleKey(name: string, roles: readonly AudiobookRoleDraft[]) {
  const base = name.trim();
  const keys = new Set(roles.map((role) => role.key));
  if (!keys.has(base)) return base;
  let suffix = 2;
  while (keys.has(`${base} ${suffix}`)) suffix += 1;
  return `${base} ${suffix}`;
}

export function renamedAudiobookDraft(
  roles: readonly AudiobookRoleDraft[],
  chapters: readonly ChapterDraft[],
  roleId: string,
  key: string,
) {
  const selected = roles.find((role) => role.id === roleId);
  if (!selected || roles.some((role) => role.id !== roleId && role.key === key)) return;
  return {
    roles: roles.map((role) => (role.id === roleId ? { ...role, key } : role)),
    chapters: chapters.map((chapter) => ({
      ...chapter,
      paragraphs: chapter.paragraphs.map((paragraph) =>
        paragraph.speakerKey === selected.key ? { ...paragraph, speakerKey: key } : paragraph,
      ),
    })),
  };
}
