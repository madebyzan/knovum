import { computed, shallowRef, watch, type DeepReadonly } from "vue";

import type { SaveSpeechProjectRevisionInput, SpeechProjectDetail, VoiceProfileSummary } from "./contracts";
import { usePronunciationLexicon } from "./use-pronunciation-lexicon";
import { defaultVoiceRevisionId } from "./voice-options";

export function useShortDubbingDraft(
  project: () => DeepReadonly<SpeechProjectDetail>,
  voices: () => readonly VoiceProfileSummary[],
) {
  const text = shallowRef("");
  const voiceRevisionId = shallowRef("");
  const saved = shallowRef("");
  const lexicon = usePronunciationLexicon(() => project().revision);

  const snapshot = () => JSON.stringify([text.value.trim(), voiceRevisionId.value, lexicon.entries.value]);
  const dirty = computed(() => snapshot() !== saved.value);
  const selectedVoice = computed(() =>
    voices().find((voice) => voice.activeRevisionId === voiceRevisionId.value),
  );
  const valid = computed(() => Boolean(text.value.trim() && selectedVoice.value && lexicon.valid.value));

  watch(
    () => project().revision.id,
    () => {
      const detail = project();
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
      outputConfig: { format: "wav" },
      segments: [
        {
          id: crypto.randomUUID(),
          text: text.value.trim(),
          voiceRevisionId: voice.activeRevisionId,
          language: voice.language,
        },
      ],
    };
  }

  return { text, voiceRevisionId, lexicon, dirty, valid, revisionInput };
}
