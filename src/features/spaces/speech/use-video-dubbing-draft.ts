import { cloneDeep } from "es-toolkit";
import { computed, ref, shallowRef, watch, type DeepReadonly } from "vue";

import type { SpeechMediaSource } from "../contracts/generated/SpeechMediaSource";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import type { SpeechSourceSeparationPublication } from "../contracts/generated/SpeechSourceSeparationPublication";
import type { VideoDubbingOutputConfig } from "../contracts/generated/VideoDubbingOutputConfig";
import type { VideoTranslation } from "../contracts/generated/VideoTranslation";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import type { SpaceMediaTranscript } from "../media/schema/transcript";
import { useDefaultVoiceProfile } from "./default-voice";
import type { DubbingSubtitle } from "./dubbing-subtitles";
import { usePronunciationLexicon } from "./use-pronunciation-lexicon";
import { useSpeechDraftChanges } from "./use-speech-draft-changes";
import {
  applyDialogueIntervals,
  selectBackground,
  selectSeparatedBackground,
} from "./video-background-selection";
import {
  applyTranslation as applyTranslatedLines,
  bindVideoSpeaker,
  videoLineIsValid,
  videoLineFromSegment,
  type VideoLineDraft,
} from "./video-dubbing-draft";
import { videoRevisionInput } from "./video-revision-input";
import { linesFromTranscript } from "./video-transcript-splitter";
import { defaultVoiceRevisionId, voiceSupportsDeliveryInstruction } from "./voice-options";
export function useVideoDubbingDraft(
  project: () => DeepReadonly<SpeechProjectSnapshot>,
  voices: () => readonly VoiceLibraryRevision[],
) {
  const { defaultVoiceProfileId } = useDefaultVoiceProfile();
  const defaultVoiceId = () => defaultVoiceRevisionId(voices(), defaultVoiceProfileId.value);
  const lines = ref<VideoLineDraft[]>([]);
  const config = shallowRef<VideoDubbingOutputConfig>();
  const voiceGain = shallowRef("0");
  const backgroundGain = shallowRef("0");
  const lexicon = usePronunciationLexicon(() => project().revision);
  const snapshot = () =>
    JSON.stringify([lines.value, config.value, voiceGain.value, backgroundGain.value, lexicon.entries.value]);
  const { dirty, needsRevision, acceptRevision, initialize } = useSpeechDraftChanges(snapshot);
  watch(
    () => project().revision.id,
    () => {
      const projectSnapshot = project();
      const outputConfig = projectSnapshot.revision.outputConfig;
      if (!("video" in outputConfig)) throw new Error("Video dubbing output configuration is required");
      config.value = cloneDeep(outputConfig) as VideoDubbingOutputConfig;
      voiceGain.value = String(config.value.voiceGainDb);
      backgroundGain.value = String(config.value.backgroundGainDb);
      lines.value = projectSnapshot.segments.map(videoLineFromSegment);
      acceptRevision();
    },
    { immediate: true },
  );
  watch(voices, () => {
    initialize(() => {
      const voiceId = defaultVoiceId();
      if (!voiceId) return;
      for (const line of lines.value) {
        if (!line.voiceRevisionId) line.voiceRevisionId = voiceId;
      }
    });
  });

  function addLine() {
    lines.value.push({
      key: crypto.randomUUID(),
      text: "",
      speakerKey: "",
      voiceRevisionId: defaultVoiceId(),
      deliveryInstruction: "",
      startSeconds: "0",
      endSeconds: "1",
    });
  }
  function importTranscript(transcript: SpaceMediaTranscript) {
    const imported = linesFromTranscript(transcript, defaultVoiceId());
    lines.value = imported.lines;
    if (config.value) config.value = applyDialogueIntervals(config.value, imported.intervals);
  }
  function initializeFromTranscript(transcript: SpaceMediaTranscript) {
    initialize(() => importTranscript(transcript));
  }
  function importSubtitles(cues: readonly DubbingSubtitle[]) {
    lines.value = cues.map((cue) => ({
      key: crypto.randomUUID(),
      text: cue.text,
      speakerKey: cue.speakerKey,
      voiceRevisionId: defaultVoiceId(),
      deliveryInstruction: "",
      startSeconds: String(cue.startMs / 1000),
      endSeconds: String(cue.endMs / 1000),
    }));
    if (config.value) config.value = applyDialogueIntervals(config.value, cues);
  }
  function bindSpeaker(speakerKey: string, voiceRevisionId: string) {
    lines.value = bindVideoSpeaker(
      lines.value,
      speakerKey,
      voiceRevisionId,
      voiceSupportsDeliveryInstruction(voices(), voiceRevisionId),
    );
  }
  function applyVoiceToAll(voiceRevisionId: string) {
    for (const line of lines.value) line.voiceRevisionId = voiceRevisionId;
  }
  function removeLine(index: number) {
    lines.value.splice(index, 1);
  }
  function updateLine(index: number, line: VideoLineDraft) {
    lines.value[index] = line;
  }
  function setBackground(background: SpeechMediaSource | null) {
    if (!config.value) throw new Error("Video dubbing output configuration is unavailable");
    config.value = selectBackground(config.value, background);
  }
  function setSeparatedBackground(publication: SpeechSourceSeparationPublication) {
    if (!config.value) throw new Error("Video dubbing output configuration is unavailable");
    config.value = selectSeparatedBackground(config.value, publication);
  }
  const valid = computed(
    () =>
      lines.value.length > 0 &&
      lines.value.every((line) => videoLineIsValid(line, voices(), config.value?.video.durationMs ?? 0)) &&
      lexicon.isValid.value &&
      [voiceGain.value, backgroundGain.value].every(
        (gain) => gain.trim() && Number.isFinite(Number(gain)) && Number(gain) >= -30 && Number(gain) <= 6,
      ),
  );
  const revisionInput = () =>
    videoRevisionInput({
      config: config.value,
      lines: lines.value,
      voices: voices(),
      lexicon: lexicon.serializeLexicon(),
      voiceGain: voiceGain.value,
      backgroundGain: backgroundGain.value,
    });
  function applyTranslation(translation: VideoTranslation) {
    lines.value = applyTranslatedLines(lines.value, translation);
    if (!config.value) throw new Error("Video dubbing output configuration is unavailable");
    config.value = { ...config.value, translation: translation.provenance };
  }
  return {
    lines,
    config,
    voiceGain,
    backgroundGain,
    lexicon,
    dirty,
    needsRevision,
    valid,
    addLine,
    removeLine,
    updateLine,
    setBackground,
    setSeparatedBackground,
    importTranscript,
    initializeFromTranscript,
    importSubtitles,
    bindSpeaker,
    applyVoiceToAll,
    applyTranslation,
    revisionInput,
  };
}
