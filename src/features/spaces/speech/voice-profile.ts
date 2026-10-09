import type { DeepReadonly } from "vue";

import type { VoiceProfileHistory } from "../contracts/generated/VoiceProfileHistory";

export function activeVoiceProfileRevision(profile: DeepReadonly<VoiceProfileHistory>) {
  const revision = profile.revisions.find(({ id }) => id === profile.activeRevisionId);
  if (!revision) throw new Error("Voice profile has no active revision");
  return revision;
}
