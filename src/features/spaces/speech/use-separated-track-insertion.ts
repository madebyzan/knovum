import { readonly, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import { useSpaceStore } from "../session/space-store";

export function useSeparatedTrackInsertion() {
  const store = useSpaceStore();
  const busy = shallowRef(false);
  const error = shallowRef<string>();
  async function insert(
    candidate: SpeechSourceSeparationCandidate,
    stem: "vocals" | "background",
    title: string,
  ) {
    if (busy.value || candidate.reviewStatus !== "accepted" || store.activeSpaceId !== candidate.spaceId)
      return;
    busy.value = true;
    error.value = undefined;
    try {
      const source = candidate[stem];
      await store.addContent({
        kind: "audio",
        title,
        fileName: stem === "vocals" ? "vocals.wav" : "accompaniment.wav",
        assetPath: source.assetPath,
        sourceRevision: source.sourceRevision,
      });
    } catch (cause) {
      if (store.activeSpaceId === candidate.spaceId) error.value = getErrorMessage(cause);
    } finally {
      busy.value = false;
    }
  }
  return { busy: readonly(busy), error: readonly(error), insert };
}
