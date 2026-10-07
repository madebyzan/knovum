import { computed, shallowRef, watch, type DeepReadonly } from "vue";

import type { SaveSpeechProjectRevisionRequest } from "../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import type { WavPrecision } from "../contracts/generated/WavPrecision";
import { usePronunciationLexicon } from "./use-pronunciation-lexicon";
import { defaultVoiceRevisionId } from "./voice-options";

export function useShortDubbingDraft(
  project: () => DeepReadonly<SpeechProjectSnapshot>,
  voices: () => readonly VoiceLibraryRevision[],
  sourceText: () => string,
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
  const valid = computed(() => Boolean(text.value.trim() && selectedVoice.value && lexicon.isValid.value));

  watch(
    () => project().revision.id,
    () => {
      const projectSnapshot = project();
      const config = projectSnapshot.revision.outputConfig;
      if ("chapters" in config || "video" in config)
        throw new Error("Short dubbing output configuration is required");
      wavPrecision.value = config.wavPrecision;
      text.value = projectSnapshot.segments.map((segment) => segment.text).join("\n\n");
      voiceRevisionId.value =
        projectSnapshot.segments[0]?.voiceRevisionId ?? defaultVoiceRevisionId(voices());
      saved.value = snapshot();
      if (projectSnapshot.project.activeRevision === 1 && !projectSnapshot.segments.length)
        text.value = sourceText();
    },
    { immediate: true },
  );

  function revisionInput(): Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision"> {
    const voice = selectedVoice.value;
    if (!voice) throw new Error("A valid voice is required");
    return {
      sourceRevision: crypto.randomUUID(),
      pronunciationLexicon: lexicon.serializeLexicon(),
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
