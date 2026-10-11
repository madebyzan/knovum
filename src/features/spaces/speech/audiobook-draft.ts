import { sumBy } from "es-toolkit";
import type { DeepReadonly } from "vue";

import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import type { ImportedAudiobookScript } from "./audiobook-script";
import { deliveryInstructionIsValid, voiceSupportsDeliveryInstruction } from "./voice-options";

interface ParagraphDraft {
  key: string;
  text: string;
  speakerKey: string;
  deliveryInstruction: string;
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
export function setAudiobookRoleVoice(
  role: AudiobookRoleDraft,
  voiceRevisionId: string,
  chapters: readonly ChapterDraft[],
  voices: readonly VoiceLibraryRevision[],
) {
  role.voiceRevisionId = voiceRevisionId;
  if (voiceSupportsDeliveryInstruction(voices, voiceRevisionId)) return;
  for (const chapter of chapters)
    for (const paragraph of chapter.paragraphs)
      if (paragraph.speakerKey === role.key) paragraph.deliveryInstruction = "";
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
  const voiceRevisionIdByRoleKey = new Map(config.roles.map((role) => [role.key, role.voiceRevisionId]));
  const chapters = config.chapters.map((chapter) => ({
    key: chapter.key,
    title: chapter.title,
    paragraphs: snapshot.segments
      .filter((segment) => segment.chapterKey === chapter.key)
      .map((segment) => {
        if (
          !segment.speakerKey ||
          voiceRevisionIdByRoleKey.get(segment.speakerKey) !== segment.voiceRevisionId
        ) {
          throw new Error("Audiobook paragraph voice differs from its role binding");
        }
        return {
          key: segment.id,
          text: segment.text,
          speakerKey: segment.speakerKey,
          deliveryInstruction: segment.deliveryInstruction ?? "",
        };
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
  const existingVoiceRevisionIdByRoleKey = new Map(
    currentRoles.map((role) => [role.key, role.voiceRevisionId]),
  );
  const roleKeys = [
    ...new Set(script.chapters.flatMap((chapter) => chapter.paragraphs.map((paragraph) => paragraph.role))),
  ];
  return {
    roles: roleKeys.map((key) => ({
      id: crypto.randomUUID(),
      key,
      voiceRevisionId: existingVoiceRevisionIdByRoleKey.get(key) ?? defaultVoiceRevisionId,
    })),
    chapters: script.chapters.map((chapter) => ({
      key: crypto.randomUUID(),
      title: chapter.title,
      paragraphs: chapter.paragraphs.map((paragraph) => ({
        key: crypto.randomUUID(),
        text: paragraph.text,
        speakerKey: paragraph.role,
        deliveryInstruction: "",
      })),
    })),
  };
}

interface AudiobookDraftValidationInput {
  roles: readonly AudiobookRoleDraft[];
  chapters: readonly ChapterDraft[];
  voices: readonly VoiceLibraryRevision[];
}

export function audiobookDraftIsValid(input: AudiobookDraftValidationInput) {
  const voiceByRevisionId = new Map(input.voices.map((voice) => [voice.revisionId, voice]));
  const roleKeys = input.roles.map((role) => role.key.trim());
  const paragraphs = input.chapters.flatMap((chapter) => chapter.paragraphs);
  const characters = sumBy(paragraphs, (paragraph) => [...paragraph.text].length);
  return (
    input.roles.length > 0 &&
    input.roles.length <= 1_000 &&
    new Set(roleKeys).size === roleKeys.length &&
    input.roles.every(
      (role, index) =>
        role.key === roleKeys[index] &&
        role.key.length > 0 &&
        [...role.key].length <= 120 &&
        voiceByRevisionId.has(role.voiceRevisionId),
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
        chapter.paragraphs.every((paragraph) => {
          const voiceRevisionId = input.roles.find(
            (role) => role.key === paragraph.speakerKey,
          )?.voiceRevisionId;
          return (
            paragraph.text.trim() &&
            roleKeys.includes(paragraph.speakerKey) &&
            deliveryInstructionIsValid(input.voices, voiceRevisionId ?? "", paragraph.deliveryInstruction)
          );
        }),
    )
  );
}

export function audiobookSegments(
  chapters: readonly ChapterDraft[],
  roles: readonly AudiobookRoleDraft[],
  voices: readonly VoiceLibraryRevision[],
) {
  const voiceByRevisionId = new Map(voices.map((voice) => [voice.revisionId, voice]));
  const voiceRevisionIdByRoleKey = new Map(roles.map((role) => [role.key, role.voiceRevisionId]));
  return chapters.flatMap((chapter) =>
    chapter.paragraphs.map((paragraph) => {
      const voiceRevisionId = voiceRevisionIdByRoleKey.get(paragraph.speakerKey);
      const voice = voiceRevisionId ? voiceByRevisionId.get(voiceRevisionId) : undefined;
      if (!voiceRevisionId || !voice) throw new Error("A valid role voice is required for every paragraph");
      const deliveryInstruction = paragraph.deliveryInstruction.trim();
      return {
        id: crypto.randomUUID(),
        chapterKey: chapter.key,
        speakerKey: paragraph.speakerKey,
        text: paragraph.text.trim(),
        voiceRevisionId,
        language: voice.language,
        ...(deliveryInstruction ? { deliveryInstruction } : {}),
      };
    }),
  );
}
