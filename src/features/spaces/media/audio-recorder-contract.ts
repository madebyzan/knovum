import type { Ref } from "vue";

import type { PersistedAudioSpaceAsset } from "./audio-recording";

export interface AudioRecorderControllerOptions {
  open: Ref<boolean>;
  spaceId: Ref<string | undefined>;
  saved: (asset: PersistedAudioSpaceAsset, spaceId: string) => void;
  failed: () => void;
  close: () => void;
}
