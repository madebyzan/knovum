import type { DeepReadonly } from "vue";

import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import type { ImportedAudiobookScript } from "./audiobook-script";

interface ParagraphDraft {
  key: string;
  text: string;
  speakerKey: string;
}
export interface ChapterDraft {
  key: string;
  title: string;
  paragraphs: ParagraphDraft[];
}
export interface AudiobookRoleDraft {
  id: string;
  key: string;
  voiceRevisionId: string;
}
export function readAudiobookRevision(snapshot: DeepReadonly<SpeechProjectSnapshot>) {
  const config = snapshot.revision.outputConfig;
  if (!("chapters" in config)) throw new Error("Audiobook output configuration is required");
  const chapterKeys = config.chapters.map((chapter) => chapter.key);
  const segmentChapterKeys = snapshot.segments.map((segment) => segment.chapterKey);
  const sequence = segmentChapterKeys.filter(
    (key, index): key is string =>
      typeof key === "string" && (index === 0 || key !== segmentChapterKeys[index - 1]),
  );
  if (
    segmentChapterKeys.some((key) => typeof key !== "string" || !chapterKeys.includes(key)) ||
    sequence.length !== chapterKeys.length ||
    sequence.some((key, index) => key !== chapterKeys[index])
  ) {
    throw new Error("Audiobook chapter order is invalid");
  }
  const roleVoices = new Map(config.roles.map((role) => [role.key, role.voiceRevisionId]));
  const chapters = config.chapters.map((chapter) => ({
    key: chapter.key,
    title: chapter.title,
    paragraphs: snapshot.segments
      .filter((segment) => segment.chapterKey === chapter.key)
      .map((segment) => {
        if (!segment.speakerKey || roleVoices.get(segment.speakerKey) !== segment.voiceRevisionId) {
          throw new Error("Audiobook paragraph voice differs from its role binding");
        }
        return { key: segment.id, text: segment.text, speakerKey: segment.speakerKey };
      }),
  }));
  return {
    config,
    chapters,
    roles: config.roles.map((role) => ({
      id: crypto.randomUUID(),
      key: role.key,
      voiceRevisionId: role.voiceRevisionId,
    })),
  };
}

export function importedAudiobookDraft(
  script: ImportedAudiobookScript,
  currentRoles: readonly AudiobookRoleDraft[],
  defaultVoiceRevisionId: string,
) {
  const existing = new Map(currentRoles.map((role) => [role.key, role.voiceRevisionId]));
  const roleKeys = [
    ...new Set(script.chapters.flatMap((chapter) => chapter.paragraphs.map((paragraph) => paragraph.role))),
  ];
  return {
    roles: roleKeys.map((key) => ({
      id: crypto.randomUUID(),
      key,
      voiceRevisionId: existing.get(key) ?? defaultVoiceRevisionId,
    })),
    chapters: script.chapters.map((chapter) => ({
      key: crypto.randomUUID(),
      title: chapter.title,
      paragraphs: chapter.paragraphs.map((paragraph) => ({
        key: crypto.randomUUID(),
        text: paragraph.text,
        speakerKey: paragraph.role,
      })),
    })),
  };
}

export function audiobookDraftIsValid(input: {
  roles: readonly AudiobookRoleDraft[];
  chapters: readonly ChapterDraft[];
  availableVoiceRevisionIds: ReadonlySet<string>;
}) {
  const roleKeys = input.roles.map((role) => role.key.trim());
  const paragraphs = input.chapters.flatMap((chapter) => chapter.paragraphs);
  const characters = paragraphs.reduce((total, paragraph) => total + [...paragraph.text].length, 0);
  return (
    input.roles.length > 0 &&
    input.roles.length <= 1_000 &&
    new Set(roleKeys).size === roleKeys.length &&
    input.roles.every(
      (role, index) =>
        role.key === roleKeys[index] &&
        role.key.length > 0 &&
        [...role.key].length <= 120 &&
        input.availableVoiceRevisionIds.has(role.voiceRevisionId),
    ) &&
    input.chapters.length > 0 &&
    input.chapters.length <= 10_000 &&
    paragraphs.length <= 10_000 &&
    characters <= 1_000_000 &&
    input.chapters.every(
      (chapter) =>
        chapter.title.trim() &&
        [...chapter.title.trim()].length <= 200 &&
        chapter.paragraphs.length > 0 &&
        chapter.paragraphs.every(
          (paragraph) => paragraph.text.trim() && roleKeys.includes(paragraph.speakerKey),
        ),
    )
  );
}

export function audiobookSegments(
  chapters: readonly ChapterDraft[],
  roles: readonly AudiobookRoleDraft[],
  voices: readonly VoiceLibraryRevision[],
) {
  const voiceLanguages = new Map(voices.map((voice) => [voice.revisionId, voice.language]));
  const roleVoices = new Map(roles.map((role) => [role.key, role.voiceRevisionId]));
  return chapters.flatMap((chapter) =>
    chapter.paragraphs.map((paragraph) => {
      const voiceRevisionId = roleVoices.get(paragraph.speakerKey);
      const language = voiceRevisionId ? voiceLanguages.get(voiceRevisionId) : undefined;
      if (!voiceRevisionId || !language)
        throw new Error("A valid role voice is required for every paragraph");
      return {
        id: crypto.randomUUID(),
        chapterKey: chapter.key,
        speakerKey: paragraph.speakerKey,
        text: paragraph.text.trim(),
        voiceRevisionId,
        language,
      };
    }),
  );
}
