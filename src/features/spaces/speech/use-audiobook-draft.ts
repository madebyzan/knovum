import { computed, ref, shallowRef, watch, type DeepReadonly } from "vue";

import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import {
  audiobookDraftIsValid,
  audiobookSegments,
  importedAudiobookDraft,
  readAudiobookRevision,
  type AudiobookRoleDraft,
  type ChapterDraft,
} from "./audiobook-draft";
import { nextAudiobookRoleKey, renamedAudiobookDraft } from "./audiobook-role-draft";
import type { ImportedAudiobookScript } from "./audiobook-script";
import { useAudiobookOutput } from "./use-audiobook-output";
import { usePronunciationLexicon } from "./use-pronunciation-lexicon";
import { defaultVoiceRevisionId } from "./voice-options";

export function useAudiobookDraft(
  project: () => DeepReadonly<SpeechProjectSnapshot>,
  voices: () => readonly VoiceLibraryRevision[],
) {
  const chapters = ref<ChapterDraft[]>([]);
  const roles = ref<AudiobookRoleDraft[]>([]);
  const output = useAudiobookOutput();
  const { format, sampleRate, loudness, peak, paragraphPause, chapterPause, wavPrecision, metadata } = output;
  const configurationError = shallowRef<string>();
  const lexicon = usePronunciationLexicon(() => project().revision);
  let saved = "";
  const snapshot = () =>
    JSON.stringify([
      chapters.value,
      roles.value,
      format.value,
      sampleRate.value,
      loudness.value,
      output.value(),
      lexicon.entries.value,
    ]);
  const dirty = computed(() => snapshot() !== saved);

  watch(
    () => project().revision.id,
    () => hydrate(project()),
    { immediate: true },
  );

  function hydrate(projectSnapshot: DeepReadonly<SpeechProjectSnapshot>) {
    try {
      const loaded = readAudiobookRevision(projectSnapshot);
      const config = loaded.config;
      output.load(config);
      roles.value = loaded.roles;
      chapters.value = loaded.chapters;
      configurationError.value = undefined;
    } catch (cause) {
      chapters.value = [];
      roles.value = [];
      configurationError.value = cause instanceof Error ? cause.message : String(cause);
    }
    saved = snapshot();
  }

  function addRole(name: string) {
    const key = nextAudiobookRoleKey(name, roles.value);
    roles.value.push({ id: crypto.randomUUID(), key, voiceRevisionId: defaultVoiceRevisionId(voices()) });
    configurationError.value = undefined;
  }
  function renameRole(role: AudiobookRoleDraft, key: string) {
    const renamed = renamedAudiobookDraft(roles.value, chapters.value, role.id, key);
    if (!renamed) return;
    roles.value = renamed.roles;
    chapters.value = renamed.chapters;
  }
  function removeRole(role: AudiobookRoleDraft) {
    if (usedRoleKeys.value.includes(role.key)) return;
    roles.value = roles.value.filter((candidate) => candidate.id !== role.id);
  }
  function addParagraph(chapter: ChapterDraft) {
    chapter.paragraphs.push({ key: crypto.randomUUID(), text: "", speakerKey: roles.value[0]?.key ?? "" });
  }
  function addChapter(title: string) {
    const chapter: ChapterDraft = { key: crypto.randomUUID(), title, paragraphs: [] };
    addParagraph(chapter);
    chapters.value.push(chapter);
    configurationError.value = undefined;
  }
  function importScript(script: ImportedAudiobookScript) {
    const imported = importedAudiobookDraft(script, roles.value, defaultVoiceRevisionId(voices()));
    roles.value = imported.roles;
    chapters.value = imported.chapters;
    configurationError.value = undefined;
  }
  const usedRoleKeys = computed(() => [
    ...new Set(
      chapters.value.flatMap((chapter) => chapter.paragraphs.map((paragraph) => paragraph.speakerKey)),
    ),
  ]);
  const selectedVoiceRevisionIds = computed(() => roles.value.map((role) => role.voiceRevisionId));
  const valid = computed(() => {
    const available = new Set(voices().map((voice) => voice.revisionId));
    return (
      !configurationError.value &&
      audiobookDraftIsValid({
        roles: roles.value,
        chapters: chapters.value,
        availableVoiceRevisionIds: available,
      }) &&
      lexicon.isValid.value &&
      output.valid() &&
      Number(loudness.value) >= -24 &&
      Number(loudness.value) <= -16 &&
      Number(peak.value) >= -3 &&
      Number(peak.value) <= -1
    );
  });

  function revisionInput() {
    return {
      sourceRevision: crypto.randomUUID(),
      pronunciationLexicon: lexicon.serializeLexicon(),
      outputConfig: {
        ...output.value(),
        chapters: chapters.value.map(({ key, title }) => ({ key, title: title.trim() })),
        roles: roles.value.map(({ key, voiceRevisionId }) => ({ key, voiceRevisionId })),
      },
      segments: audiobookSegments(chapters.value, roles.value, voices()),
    };
  }
  function roleVoiceRevisionId(speakerKey: string) {
    return roles.value.find((role) => role.key === speakerKey)?.voiceRevisionId ?? "";
  }
  function moveChapter(index: number, delta: number) {
    const destination = index + delta;
    if (destination < 0 || destination >= chapters.value.length) return;
    const [chapter] = chapters.value.splice(index, 1);
    if (chapter) chapters.value.splice(destination, 0, chapter);
  }
  function removeChapter(index: number) {
    chapters.value.splice(index, 1);
  }
  return {
    chapters,
    roles,
    format,
    sampleRate,
    loudness,
    peak,
    paragraphPause,
    chapterPause,
    wavPrecision,
    metadata,
    lexicon,
    configurationError,
    dirty,
    valid,
    usedRoleKeys,
    selectedVoiceRevisionIds,
    addRole,
    renameRole,
    removeRole,
    addChapter,
    addParagraph,
    moveChapter,
    removeChapter,
    removeParagraph,
    importScript,
    roleVoiceRevisionId,
    revisionInput,
  };
}

function removeParagraph(chapter: ChapterDraft, index: number) {
  chapter.paragraphs.splice(index, 1);
}
