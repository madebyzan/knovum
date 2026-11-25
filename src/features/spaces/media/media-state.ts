import { ref } from "vue";

import type { SpaceMediaAnalysisPreview } from "./analysis-preview";
import type { SpaceAudioRecordingTarget } from "./audio-recording-actions";
import type { SpaceMediaPlaybackState, SpaceMediaSeekRequest, SpaceMediaToggleRequest } from "./playback";

export function createSpaceMediaState() {
  return {
    audioRecordingTarget: ref<SpaceAudioRecordingTarget>(),
    mediaAnalysisIds: ref<string[]>([]),
    mediaAnalysisPreviews: ref<Record<string, SpaceMediaAnalysisPreview>>({}),
    mediaTranscriptionLanguageByNodeId: ref<Record<string, string>>({}),
    mediaSeekRequest: ref<SpaceMediaSeekRequest>(),
    mediaToggleRequest: ref<SpaceMediaToggleRequest>(),
    mediaPlayback: ref<{ id: string; state: SpaceMediaPlaybackState }>(),
  };
}
