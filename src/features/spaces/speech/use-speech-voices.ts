import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import { createPresetVoice, listVoiceRevisions } from "./ipc";
import { VOICE_QUERY_KEYS } from "./voice-query-keys";

export function useSpeechVoices() {
  const client = useQueryClient();
  const query = useQuery({ queryKey: VOICE_QUERY_KEYS.revisions, queryFn: listVoiceRevisions });
  const creation = useMutation({
    mutationFn: createPresetVoice,
    onSuccess: async (voice) => {
      client.setQueryData<VoiceLibraryRevision[]>(VOICE_QUERY_KEYS.revisions, (current) => [
        voice,
        ...(current ?? []).filter(({ revisionId }) => revisionId !== voice.revisionId),
      ]);
      await client.invalidateQueries({ queryKey: VOICE_QUERY_KEYS.profiles });
    },
  });

  return {
    voices: computed(() => query.data.value ?? []),
    loading: query.isPending,
    creating: creation.isPending,
    errors: computed(() =>
      [creation.error.value, query.error.value]
        .filter((cause): cause is Error => Boolean(cause))
        .map(getErrorMessage),
    ),
    addPreset: creation.mutate,
    addPresetAsync: creation.mutateAsync,
  };
}
