import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { CreatePresetVoiceRequest } from "../contracts/generated/CreatePresetVoiceRequest";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import { createPresetVoice, listVoiceRevisions } from "./ipc";
import { voiceQueryKeys } from "./voice-query-keys";

export function useSpeechVoices(setError?: (message?: string) => void) {
  const client = useQueryClient();
  const query = useQuery({ queryKey: voiceQueryKeys.revisions, queryFn: listVoiceRevisions });
  const creation = useMutation({
    mutationFn: createPresetVoice,
    onSuccess: async (voice) => {
      client.setQueryData<VoiceLibraryRevision[]>(voiceQueryKeys.revisions, (current) => [
        voice,
        ...(current ?? []).filter(({ revisionId }) => revisionId !== voice.revisionId),
      ]);
      await client.invalidateQueries({ queryKey: voiceQueryKeys.profiles });
      setError?.();
    },
    onError: (cause) => setError?.(getErrorMessage(cause)),
  });
  watch(query.error, (cause) => {
    if (cause) setError?.(getErrorMessage(cause));
  });

  return {
    voices: computed(() => query.data.value ?? []),
    loading: query.isPending,
    creating: creation.isPending,
    error: computed(() => {
      const cause = creation.error.value ?? query.error.value;
      return cause ? getErrorMessage(cause) : undefined;
    }),
    addPreset: (input: CreatePresetVoiceRequest) => creation.mutate(input),
    addPresetAsync: (input: CreatePresetVoiceRequest) => creation.mutateAsync(input),
  };
}
