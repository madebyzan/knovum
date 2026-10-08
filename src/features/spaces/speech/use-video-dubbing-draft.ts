import { cloneDeep } from "es-toolkit";
import { computed, ref, shallowRef, watch, type DeepReadonly } from "vue";

import type { SpeechMediaSource } from "../contracts/generated/SpeechMediaSource";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import type { VideoDubbingOutputConfig } from "../contracts/generated/VideoDubbingOutputConfig";
import type { VideoTranslation } from "../contracts/generated/VideoTranslation";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import type { SpaceMediaTranscript } from "../media/schema/transcript";
import type { DubbingSubtitle } from "./dubbing-subtitles";
import { usePronunciationLexicon } from "./use-pronunciation-lexicon";
import { selectBackground, selectSeparatedBackground } from "./video-background-selection";
import {
  applyTranslation as applyTranslatedLines,
  bindVideoSpeaker,
  translationInputs as buildTranslationInputs,
  videoLineIsValid,
  videoLineFromSegment,
  type VideoLineDraft,
} from "./video-dubbing-draft";
import { videoRevisionInput } from "./video-revision-input";
import { defaultVoiceRevisionId, voiceSupportsDeliveryInstruction } from "./voice-options";
export function useVideoDubbingDraft(
  project: () => DeepReadonly<SpeechProjectSnapshot>,
  voices: () => readonly VoiceLibraryRevision[],
) {
  const lines = ref<VideoLineDraft[]>([]);
  const config = shallowRef<VideoDubbingOutputConfig>();
  const voiceGain = shallowRef("0");
  const backgroundGain = shallowRef("-12");
  const lexicon = usePronunciationLexicon(() => project().revision);
  let saved = "";
  const snapshot = () =>
    JSON.stringify([lines.value, config.value, voiceGain.value, backgroundGain.value, lexicon.entries.value]);
  const dirty = computed(() => snapshot() !== saved);
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
      saved = snapshot();
    },
    { immediate: true },
  );
  function addLine() {
    lines.value.push({
      key: crypto.randomUUID(),
      text: "",
      speakerKey: "",
      voiceRevisionId: defaultVoiceRevisionId(voices()),
      deliveryInstruction: "",
      startSeconds: "0",
      endSeconds: "1",
      needsTimingEdit: false,
    });
  }
  function importTranscript(transcript: SpaceMediaTranscript) {
    lines.value = transcript.segments.map((segment) => ({
      key: crypto.randomUUID(),
      text: segment.text,
      speakerKey: segment.speakerId ?? "",
      voiceRevisionId: defaultVoiceRevisionId(voices()),
      deliveryInstruction: "",
      startSeconds: String(segment.startMs / 1000),
      endSeconds: String(segment.endMs / 1000),
      needsTimingEdit: false,
      sourceSegmentId: segment.id,
      sourceSegmentRevision: segment.revision,
      sourceText: segment.text,
    }));
    if (config.value)
      config.value = {
        ...config.value,
        translation: null,
        sourceDialogueIntervals: transcript.segments.map(({ startMs, endMs }) => ({ startMs, endMs })),
      };
  }
  function importSubtitles(cues: readonly DubbingSubtitle[]) {
    lines.value = cues.map((cue) => ({
      key: crypto.randomUUID(),
      text: cue.text,
      speakerKey: cue.speakerKey,
      voiceRevisionId: defaultVoiceRevisionId(voices()),
      deliveryInstruction: "",
      startSeconds: String(cue.startMs / 1000),
      endSeconds: String(cue.endMs / 1000),
      needsTimingEdit: false,
    }));
    if (config.value)
      config.value = {
        ...config.value,
        translation: null,
        sourceDialogueIntervals: cues.map(({ startMs, endMs }) => ({ startMs, endMs })),
      };
  }
  function bindSpeaker(speakerKey: string, voiceRevisionId: string) {
    lines.value = bindVideoSpeaker(
      lines.value,
      speakerKey,
      voiceRevisionId,
      voiceSupportsDeliveryInstruction(voices(), voiceRevisionId),
    );
  }
  function removeLine(index: number) {
    lines.value.splice(index, 1);
  }
  function setBackground(background: SpeechMediaSource | null) {
    if (!config.value) throw new Error("Video dubbing output configuration is unavailable");
    config.value = selectBackground(config.value, background);
  }
  function setSeparatedBackground(candidate: SpeechSourceSeparationCandidate) {
    if (!config.value) throw new Error("Video dubbing output configuration is unavailable");
    config.value = selectSeparatedBackground(config.value, candidate);
  }
  function updateLine(index: number, line: VideoLineDraft) {
    lines.value[index] = line;
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
  const translationInputs = () => buildTranslationInputs(lines.value, Boolean(config.value?.translation));
  function applyTranslation(translation: VideoTranslation) {
    lines.value = applyTranslatedLines(lines.value, translation, Boolean(config.value?.translation));
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
    valid,
    addLine,
    removeLine,
    updateLine,
    setBackground,
    setSeparatedBackground,
    importTranscript,
    importSubtitles,
    bindSpeaker,
    translationInputs,
    applyTranslation,
    revisionInput,
  };
}
