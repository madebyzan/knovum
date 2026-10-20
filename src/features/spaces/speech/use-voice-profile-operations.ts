import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, type DeepReadonly } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { VoiceProfileHistory } from "../contracts/generated/VoiceProfileHistory";
import {
  archiveVoiceProfile,
  deleteVoiceProfile,
  listVoiceProfiles,
  renameVoiceProfile,
  reviseVoiceProfile,
  unarchiveVoiceProfile,
} from "./ipc";
import { activeVoiceProfileRevision } from "./voice-profile";
import { VOICE_QUERY_KEYS } from "./voice-query-keys";

interface ProfileOperation {
  profileId: string;
  execute: () => Promise<unknown>;
}

export function useVoiceProfileOperations() {
  const client = useQueryClient();
  const query = useQuery({
    queryKey: VOICE_QUERY_KEYS.profiles,
    queryFn: async () => {
      const profiles = await listVoiceProfiles();
      profiles.forEach(activeVoiceProfileRevision);
      return profiles;
    },
  });
  const mutation = useMutation({
    mutationFn: ({ execute }: ProfileOperation) => execute(),
    onSettled: async () => {
      await Promise.all([
        client.invalidateQueries({ queryKey: VOICE_QUERY_KEYS.profiles }),
        client.invalidateQueries({ queryKey: VOICE_QUERY_KEYS.revisions }),
      ]);
    },
  });

  function mutate(profileId: string, execute: () => Promise<unknown>) {
    if (!mutation.isPending.value) mutation.mutate({ profileId, execute });
  }

  return {
    reload: query.refetch,
    profiles: computed(() => query.data.value ?? []),
    loading: query.isPending,
    busyProfileId: computed(() =>
      mutation.isPending.value ? mutation.variables.value?.profileId : undefined,
    ),
    errors: computed(() =>
      [mutation.error.value, query.error.value]
        .filter((cause): cause is Error => Boolean(cause))
        .map(getErrorMessage),
    ),
    rename: (profile: DeepReadonly<VoiceProfileHistory>, name: string) =>
      mutate(profile.id, () =>
        renameVoiceProfile({ profileId: profile.id, expectedUpdatedAt: profile.updatedAt, name }),
      ),
    revise: (profile: DeepReadonly<VoiceProfileHistory>, language: string, speed: number) =>
      mutate(profile.id, () =>
        reviseVoiceProfile({
          profileId: profile.id,
          expectedActiveRevisionId: profile.activeRevisionId,
          language,
          speed,
        }),
      ),
    archive: (profile: DeepReadonly<VoiceProfileHistory>) =>
      mutate(profile.id, () =>
        archiveVoiceProfile({ profileId: profile.id, expectedUpdatedAt: profile.updatedAt }),
      ),
    unarchive: (profile: DeepReadonly<VoiceProfileHistory>) =>
      mutate(profile.id, () =>
        unarchiveVoiceProfile({ profileId: profile.id, expectedUpdatedAt: profile.updatedAt }),
      ),
    remove: (profile: DeepReadonly<VoiceProfileHistory>) =>
      mutate(profile.id, () =>
        deleteVoiceProfile({ profileId: profile.id, expectedUpdatedAt: profile.updatedAt }),
      ),
  };
}
