import type { Ref } from "vue";

import type { PersistedAudioSpaceAsset } from "./audio-recording";

export interface AudioRecorderControllerOptions {
  open: Ref<boolean>;
  spaceId: Ref<string | undefined>;
  save: (asset: PersistedAudioSpaceAsset, spaceId: string) => Promise<void>;
  failed: () => void;
  close: () => void;
}
