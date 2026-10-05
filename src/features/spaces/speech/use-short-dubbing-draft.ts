import { computed, shallowRef, watch, type DeepReadonly } from "vue";

import type { WavPrecision } from "../contracts/generated/WavPrecision";
import type { SaveSpeechProjectRevisionInput, SpeechProjectDetail, VoiceLibraryRevision } from "./contracts";
import { usePronunciationLexicon } from "./use-pronunciation-lexicon";
import { defaultVoiceRevisionId } from "./voice-options";

export function useShortDubbingDraft(
  project: () => DeepReadonly<SpeechProjectDetail>,
  voices: () => readonly VoiceLibraryRevision[],
) {
  const text = shallowRef("");
  const voiceRevisionId = shallowRef("");
  const wavPrecision = shallowRef<WavPrecision>("pcm16");
  const saved = shallowRef("");
  const lexicon = usePronunciationLexicon(() => project().revision);

  const snapshot = () =>
    JSON.stringify([text.value.trim(), voiceRevisionId.value, wavPrecision.value, lexicon.entries.value]);
  const dirty = computed(() => snapshot() !== saved.value);
  const selectedVoice = computed(() => voices().find((voice) => voice.revisionId === voiceRevisionId.value));
  const valid = computed(() => Boolean(text.value.trim() && selectedVoice.value && lexicon.valid.value));

  watch(
    () => project().revision.id,
    () => {
      const detail = project();
      wavPrecision.value = (
        JSON.parse(detail.revision.outputConfig) as { wavPrecision: typeof wavPrecision.value }
      ).wavPrecision;
      text.value = detail.segments.map((segment) => segment.text).join("\n\n");
      voiceRevisionId.value = detail.segments[0]?.voiceRevisionId ?? defaultVoiceRevisionId(voices());
      saved.value = snapshot();
    },
    { immediate: true },
  );

  function revisionInput(): Omit<SaveSpeechProjectRevisionInput, "projectId" | "expectedRevision"> {
    const voice = selectedVoice.value;
    if (!voice) throw new Error("A valid voice is required");
    return {
      sourceRevision: crypto.randomUUID(),
      pronunciationLexicon: lexicon.value(),
      outputConfig: { format: "wav", wavPrecision: wavPrecision.value },
      segments: [
        {
          id: crypto.randomUUID(),
          text: text.value.trim(),
          voiceRevisionId: voice.revisionId,
          language: voice.language,
        },
      ],
    };
  }

  return { text, voiceRevisionId, wavPrecision, lexicon, dirty, valid, revisionInput };
}
