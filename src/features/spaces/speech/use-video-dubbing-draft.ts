import { computed, ref, shallowRef, watch, type DeepReadonly } from "vue";

import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import type { VideoTranslationResult } from "../contracts/generated/VideoTranslationResult";
import type { SpaceMediaTranscript } from "../media/schema/transcript";
import type {
  SpeechMediaSource,
  SpeechProjectDetail,
  VideoDubbingOutputConfig,
  VoiceProfileSummary,
} from "./contracts";
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
import { defaultVoiceRevisionId } from "./voice-options";
export function useVideoDubbingDraft(
  project: () => DeepReadonly<SpeechProjectDetail>,
  voices: () => readonly VoiceProfileSummary[],
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
    if (config.value) config.value = { ...config.value, translation: null };
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
          voices().some((voice) => voice.activeRevisionId === line.voiceRevisionId) &&
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
  function revisionInput() {
    const currentConfig = config.value;
    if (!currentConfig) throw new Error("Video dubbing output configuration is unavailable");
    const voiceLanguages = new Map(voices().map((voice) => [voice.activeRevisionId, voice.language]));
    return {
      sourceRevision: crypto.randomUUID(),
      pronunciationLexicon: lexicon.value(),
      outputConfig: {
        ...currentConfig,
        voiceGainDb: Number(voiceGain.value),
        backgroundGainDb: Number(backgroundGain.value),
      },
      segments: lines.value.map((line) => {
        const language = voiceLanguages.get(line.voiceRevisionId);
        if (!language) throw new Error("A valid voice is required for every timed line");
        const speakerKey = line.speakerKey.trim();
        return {
          id: crypto.randomUUID(),
          text: line.text.trim(),
          ...(speakerKey ? { speakerKey } : {}),
          voiceRevisionId: line.voiceRevisionId,
          language,
          targetStartMs: milliseconds(line.startSeconds),
          targetEndMs: milliseconds(line.endSeconds),
          ...(line.sourceSegmentId && line.sourceSegmentRevision && line.sourceText
            ? {
                sourceSegmentId: line.sourceSegmentId,
                sourceSegmentRevision: line.sourceSegmentRevision,
                sourceText: line.sourceText,
              }
            : {}),
        };
      }),
    };
  }
  const translationInputs = () => buildTranslationInputs(lines.value, Boolean(config.value?.translation));
  function applyTranslation(result: VideoTranslationResult) {
    const inputs = buildTranslationInputs(lines.value, Boolean(config.value?.translation));
    if (
      inputs.length !== result.segments.length ||
      inputs.some((input, index) => {
        const output = result.segments[index];
        return (
          !output ||
          output.sourceSegmentId !== input.sourceSegmentId ||
          output.sourceSegmentRevision !== input.sourceSegmentRevision ||
          output.sourceText !== input.text.trim()
        );
      })
    ) {
      throw new Error("Translation result no longer matches the editable video script");
    }
    lines.value = applyTranslatedLines(lines.value, result);
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
    translationInputs,
    applyTranslation,
    revisionInput,
  };
}
