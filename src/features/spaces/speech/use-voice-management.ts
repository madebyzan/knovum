import { onScopeDispose, readonly, shallowRef, watch, type DeepReadonly } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { VoiceProfileHistory } from "../contracts/generated/VoiceProfileHistory";
import {
  copyVoiceProfile,
  deleteVoiceProfile,
  listVoiceProfiles,
  renameVoiceProfile,
  reviseVoiceProfile,
  archiveVoiceProfile,
  unarchiveVoiceProfile,
} from "./ipc";

export function useVoiceManagement(ownerSpaceId: () => string | undefined, changed: () => void) {
  const profiles = shallowRef<VoiceProfileHistory[]>([]);
  const busyProfileId = shallowRef<string>();
  const error = shallowRef<string>();
  let epoch = 0;
  let contextEpoch = 0;

  function current(spaceId: string | undefined, version: number) {
    return ownerSpaceId() === spaceId && contextEpoch === version;
  }

  async function load() {
    const spaceId = ownerSpaceId();
    const requestVersion = ++epoch;
    if (!spaceId) return void (profiles.value = []);
    error.value = undefined;
    try {
      const next = await listVoiceProfiles(spaceId);
      if (requestVersion === epoch && ownerSpaceId() === spaceId) profiles.value = next;
    } catch (cause) {
      if (requestVersion === epoch) error.value = getErrorMessage(cause);
    }
  }

  async function mutate(profileId: string, operation: () => Promise<unknown>) {
    if (busyProfileId.value) return;
    const spaceId = ownerSpaceId();
    const version = contextEpoch;
    busyProfileId.value = profileId;
    error.value = undefined;
    try {
      await operation();
      if (!current(spaceId, version)) return;
      await load();
      if (current(spaceId, version)) changed();
    } catch (cause) {
      if (current(spaceId, version)) error.value = getErrorMessage(cause);
    } finally {
      if (current(spaceId, version)) busyProfileId.value = undefined;
    }
  }

  watch(
    () => ownerSpaceId(),
    () => {
      contextEpoch += 1;
      profiles.value = [];
      busyProfileId.value = undefined;
      error.value = undefined;
      void load();
    },
    { immediate: true },
  );
  onScopeDispose(() => {
    epoch += 1;
    contextEpoch += 1;
  });
  return {
    reload: load,
    profiles: readonly(profiles),
    busyProfileId: readonly(busyProfileId),
    error: readonly(error),
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
        archiveVoiceProfile({
          profileId: profile.id,
          expectedUpdatedAt: profile.updatedAt,
        }),
      ),
    unarchive: (profile: DeepReadonly<VoiceProfileHistory>) =>
      mutate(profile.id, () =>
        unarchiveVoiceProfile({ profileId: profile.id, expectedUpdatedAt: profile.updatedAt }),
      ),
    remove: (profile: DeepReadonly<VoiceProfileHistory>) =>
      mutate(profile.id, () =>
        deleteVoiceProfile({ profileId: profile.id, expectedUpdatedAt: profile.updatedAt }),
      ),
    copy: (profile: DeepReadonly<VoiceProfileHistory>, targetSpaceId: string, name: string) =>
      mutate(profile.id, () =>
        copyVoiceProfile({
          profileId: profile.id,
          expectedActiveRevisionId: profile.activeRevisionId,
          targetSpaceId,
          name,
        }),
      ),
  };
}
