import { computed, ref, shallowRef, watch, type DeepReadonly } from "vue";

import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import type { VideoTranslationResult } from "../contracts/generated/VideoTranslationResult";
import type { SpaceMediaTranscript } from "../media/schema/transcript";
import type {
  SpeechMediaSource,
  SpeechProjectDetail,
  VideoDubbingOutputConfig,
  VoiceLibraryRevision,
} from "./contracts";
import type { DubbingSubtitle } from "./dubbing-subtitles";
import { usePronunciationLexicon } from "./use-pronunciation-lexicon";
import { selectBackground, selectSeparatedBackground } from "./video-background-selection";
import {
  applyTranslation as applyTranslatedLines,
  milliseconds,
  translationInputs as buildTranslationInputs,
  validVideoWindow,
  videoLineFromSegment,
  type VideoLineDraft,
} from "./video-dubbing-draft";
import { videoRevisionInput } from "./video-revision-input";
import { defaultVoiceRevisionId } from "./voice-options";
export function useVideoDubbingDraft(
  project: () => DeepReadonly<SpeechProjectDetail>,
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
      const detail = project();
      config.value = JSON.parse(detail.revision.outputConfig) as VideoDubbingOutputConfig;
      voiceGain.value = String(config.value.voiceGainDb);
      backgroundGain.value = String(config.value.backgroundGainDb);
      lines.value = detail.segments.map(videoLineFromSegment);
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
    lines.value = lines.value.map((line) =>
      line.speakerKey.trim() === speakerKey ? { ...line, voiceRevisionId } : line,
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
      lines.value.every((line) => {
        const start = milliseconds(line.startSeconds);
        const end = milliseconds(line.endSeconds);
        return (
          line.text.trim() &&
          voices().some((voice) => voice.revisionId === line.voiceRevisionId) &&
          line.startSeconds.trim() &&
          line.endSeconds.trim() &&
          validVideoWindow(start, end, config.value?.video.durationMs ?? 0)
        );
      }) &&
      lexicon.valid.value &&
      [voiceGain.value, backgroundGain.value].every(
        (gain) => gain.trim() && Number.isFinite(Number(gain)) && Number(gain) >= -30 && Number(gain) <= 6,
      ),
  );
  const revisionInput = () =>
    videoRevisionInput({
      config: config.value,
      lines: lines.value,
      voices: voices(),
      lexicon: lexicon.value(),
      voiceGain: voiceGain.value,
      backgroundGain: backgroundGain.value,
    });
  const translationInputs = () => buildTranslationInputs(lines.value, Boolean(config.value?.translation));
  function applyTranslation(result: VideoTranslationResult) {
    lines.value = applyTranslatedLines(lines.value, result, Boolean(config.value?.translation));
    if (!config.value) throw new Error("Video dubbing output configuration is unavailable");
    config.value = { ...config.value, translation: result.provenance };
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
