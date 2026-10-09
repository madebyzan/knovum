import type { DeepReadonly } from "vue";

import type { SpeechAudioOutput } from "../contracts/generated/SpeechAudioOutput";
import type { SpeechProjectRevision } from "../contracts/generated/SpeechProjectRevision";

export function publishedSpeechOutput(
  revision?: DeepReadonly<SpeechProjectRevision>,
): SpeechAudioOutput | undefined {
  if (
    !revision?.outputAssetPath ||
    !revision.outputSha256 ||
    revision.outputSampleRate === undefined ||
    revision.outputSampleCount === undefined
  )
    return undefined;
  return {
    path: revision.outputAssetPath,
    sha256: revision.outputSha256,
    sampleRate: revision.outputSampleRate,
    sampleCount: revision.outputSampleCount,
  };
}
