import type { DeepReadonly } from "vue";

import type { ImportedAudiobookScript } from "./audiobook-script-import";
import type { AudiobookOutputConfig, SpeechProjectDetail, VoiceProfileSummary } from "./contracts";

export interface ParagraphDraft {
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
export function readAudiobookRevision(detail: DeepReadonly<SpeechProjectDetail>) {
  const config = JSON.parse(detail.revision.outputConfig) as AudiobookOutputConfig;
  if (!Array.isArray(config.roles)) throw new Error("Audiobook role bindings are missing");
  const roleKeys = config.roles.map((role) => role.key);
  if (
    config.roles.some(
      (role) =>
        typeof role.key !== "string" ||
        !role.key ||
        role.key !== role.key.trim() ||
        [...role.key].length > 120 ||
        typeof role.voiceRevisionId !== "string" ||
        !role.voiceRevisionId,
    ) ||
    new Set(roleKeys).size !== roleKeys.length
  ) {
    throw new Error("Audiobook role bindings are invalid");
  }
  const chapterKeys = config.chapters.map((chapter) => chapter.key);
  const segmentChapterKeys = detail.segments.map((segment) => segment.chapterKey);
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
    paragraphs: detail.segments
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
    roles: config.roles.map((role) => ({ id: crypto.randomUUID(), ...role })),
    voiceLanguages: new Map(detail.segments.map((segment) => [segment.voiceRevisionId, segment.language])),
    voiceRevisionIds: new Set(config.roles.map((role) => role.voiceRevisionId)),
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
  voices: readonly VoiceProfileSummary[],
  savedVoiceLanguages: ReadonlyMap<string, string>,
) {
  const voiceLanguages = new Map(voices.map((voice) => [voice.activeRevisionId, voice.language]));
  const roleVoices = new Map(roles.map((role) => [role.key, role.voiceRevisionId]));
  return chapters.flatMap((chapter) =>
    chapter.paragraphs.map((paragraph) => {
      const voiceRevisionId = roleVoices.get(paragraph.speakerKey);
      const language = voiceRevisionId
        ? (voiceLanguages.get(voiceRevisionId) ?? savedVoiceLanguages.get(voiceRevisionId))
        : undefined;
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
