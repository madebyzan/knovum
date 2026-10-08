import { computed, shallowRef, watch, type DeepReadonly } from "vue";

import type { SaveSpeechProjectRevisionRequest } from "../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import type { WavPrecision } from "../contracts/generated/WavPrecision";
import { usePronunciationLexicon } from "./use-pronunciation-lexicon";
import {
  defaultVoiceRevisionId,
  deliveryInstructionIsValid,
  voiceSupportsDeliveryInstruction,
} from "./voice-options";

export function useShortDubbingDraft(
  project: () => DeepReadonly<SpeechProjectSnapshot>,
  voices: () => readonly VoiceLibraryRevision[],
  sourceText: () => string,
) {
  const text = shallowRef("");
  const voiceRevisionId = shallowRef("");
  const deliveryInstruction = shallowRef("");
  const wavPrecision = shallowRef<WavPrecision>("pcm16");
  const saved = shallowRef("");
  const lexicon = usePronunciationLexicon(() => project().revision);

  const snapshot = () =>
    JSON.stringify([
      text.value.trim(),
      voiceRevisionId.value,
      deliveryInstruction.value.trim(),
      wavPrecision.value,
      lexicon.entries.value,
    ]);
  const dirty = computed(() => snapshot() !== saved.value);
  const selectedVoice = computed(() => voices().find((voice) => voice.revisionId === voiceRevisionId.value));
  const supportsDeliveryInstruction = computed(() =>
    voiceSupportsDeliveryInstruction(voices(), voiceRevisionId.value),
  );
  const valid = computed(
    () =>
      Boolean(text.value.trim() && selectedVoice.value && lexicon.isValid.value) &&
      deliveryInstructionIsValid(voices(), voiceRevisionId.value, deliveryInstruction.value),
  );

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
      deliveryInstruction.value = projectSnapshot.segments[0]?.deliveryInstruction ?? "";
      saved.value = snapshot();
      if (projectSnapshot.project.activeRevision === 1 && !projectSnapshot.segments.length)
        text.value = sourceText();
    },
    { immediate: true },
  );

  watch(voices, () => {
    if (!voiceRevisionId.value && !project().segments.length)
      voiceRevisionId.value = defaultVoiceRevisionId(voices());
  });
  watch(supportsDeliveryInstruction, (supported) => {
    if (!supported) deliveryInstruction.value = "";
  });

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
          ...(deliveryInstruction.value.trim()
            ? { deliveryInstruction: deliveryInstruction.value.trim() }
            : {}),
        },
      ],
    };
  }

  return {
    text,
    voiceRevisionId,
    deliveryInstruction,
    supportsDeliveryInstruction,
    wavPrecision,
    lexicon,
    dirty,
    valid,
    revisionInput,
  };
}
