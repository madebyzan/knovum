import { shallowRef } from "vue";

import { useSpeechPreview } from "./use-speech-preview";

/**
 * Auditioning voices from the library, where exactly one voice may be heard at a time.
 *
 * A voice has no stored sample — a preset is an engine, a language and a speed, not a recording —
 * so pressing play synthesises a short line with that revision and streams it. Starting another
 * row therefore stops the current one instead of layering two voices over each other.
 */
export function useVoiceAudition(sampleText: () => string) {
  const preview = useSpeechPreview();
  const auditioningId = shallowRef<string>();

  async function audition(profileId: string, revisionId: string) {
    if (auditioningId.value === profileId) {
      stop();
      return;
    }
    stop();
    auditioningId.value = profileId;
    try {
      await preview.previewSelection(sampleText(), revisionId, {});
    } finally {
      if (auditioningId.value === profileId) auditioningId.value = undefined;
    }
  }

  function stop() {
    auditioningId.value = undefined;
    preview.reset();
  }

  return {
    error: preview.error,
    stop,
    audition,
    stateFor: (profileId: string) => ({
      playing: auditioningId.value === profileId,
      busy: auditioningId.value !== undefined && auditioningId.value !== profileId,
      peak: preview.peak,
    }),
  };
}
