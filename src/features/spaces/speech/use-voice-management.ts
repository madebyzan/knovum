import { readonly, shallowRef, watch, type DeepReadonly } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { VoiceProfileHistory } from "../contracts/generated/VoiceProfileHistory";
import { useSpaceStore } from "../session/space-store";
import {
  copyVoiceProfile,
  deleteVoiceProfile,
  listVoiceProfiles,
  renameVoiceProfile,
  reviseVoiceProfile,
  archiveVoiceProfile,
  unarchiveVoiceProfile,
} from "./ipc";

export function useVoiceManagement(changed: () => void) {
  const store = useSpaceStore();
  const profiles = shallowRef<VoiceProfileHistory[]>([]);
  const busyProfileId = shallowRef<string>();
  const error = shallowRef<string>();
  let epoch = 0;

  async function load() {
    const spaceId = store.activeSpaceId;
    const current = ++epoch;
    if (!spaceId) return void (profiles.value = []);
    try {
      const next = await listVoiceProfiles(spaceId);
      if (current === epoch && store.activeSpaceId === spaceId) profiles.value = next;
    } catch (cause) {
      if (current === epoch) error.value = getErrorMessage(cause);
    }
  }

  async function mutate(profileId: string, operation: () => Promise<unknown>) {
    if (busyProfileId.value) return;
    busyProfileId.value = profileId;
    try {
      await operation();
      error.value = undefined;
      await load();
      changed();
    } catch (cause) {
      error.value = getErrorMessage(cause);
    } finally {
      busyProfileId.value = undefined;
    }
  }

  watch(() => store.activeSpaceId, load, { immediate: true });
  return {
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
