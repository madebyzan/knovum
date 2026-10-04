import { readonly, shallowRef, watch, type DeepReadonly } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useSpaceStore } from "../session/space-store";
import type { VoiceProfileDetail } from "./contracts";
import {
  copyVoiceProfile,
  deleteVoiceProfile,
  listVoiceProfilesForManagement,
  renameVoiceProfile,
  reviseVoiceProfile,
  setVoiceProfileArchived,
} from "./ipc";

export function useVoiceManagement(changed: () => void) {
  const store = useSpaceStore();
  const profiles = shallowRef<VoiceProfileDetail[]>([]);
  const busyProfileId = shallowRef<string>();
  const error = shallowRef<string>();
  let epoch = 0;

  async function load() {
    const spaceId = store.activeSpaceId;
    const current = ++epoch;
    if (!spaceId) return void (profiles.value = []);
    try {
      const next = await listVoiceProfilesForManagement(spaceId);
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
    rename: (profile: DeepReadonly<VoiceProfileDetail>, name: string) =>
      mutate(profile.id, () =>
        renameVoiceProfile({ profileId: profile.id, expectedUpdatedAt: profile.updatedAt, name }),
      ),
    revise: (profile: DeepReadonly<VoiceProfileDetail>, language: string, speed: number) =>
      mutate(profile.id, () =>
        reviseVoiceProfile({
          profileId: profile.id,
          expectedActiveRevisionId: profile.activeRevisionId,
          language,
          speed,
        }),
      ),
    archive: (profile: DeepReadonly<VoiceProfileDetail>, archived: boolean) =>
      mutate(profile.id, () =>
        setVoiceProfileArchived({
          profileId: profile.id,
          expectedUpdatedAt: profile.updatedAt,
          archived,
        }),
      ),
    remove: (profile: DeepReadonly<VoiceProfileDetail>) =>
      mutate(profile.id, () =>
        deleteVoiceProfile({ profileId: profile.id, expectedUpdatedAt: profile.updatedAt }),
      ),
    copy: (profile: DeepReadonly<VoiceProfileDetail>, targetSpaceId: string, name: string) =>
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
