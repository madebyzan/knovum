import { shallowRef } from "vue";

import type { AudiobookMetadata } from "../contracts/generated/AudiobookMetadata";
import type { AudiobookOutputConfig } from "../contracts/generated/AudiobookOutputConfig";
import type { WavPrecision } from "../contracts/generated/WavPrecision";

export function useAudiobookOutput() {
  const format = shallowRef<"wav" | "mp3" | "m4b">("m4b");
  const sampleRate = shallowRef<"24000" | "48000">("48000");
  const loudness = shallowRef("-20");
  const peak = shallowRef("-2");
  const paragraphPause = shallowRef("250");
  const chapterPause = shallowRef("1000");
  const wavPrecision = shallowRef<WavPrecision>("pcm16");
  const metadata = shallowRef<AudiobookMetadata>({ title: "", author: "", narrator: "", cover: null });
  function load(config: AudiobookOutputConfig) {
    format.value = config.format as "wav" | "mp3" | "m4b";
    sampleRate.value = String(config.sampleRate) as "24000" | "48000";
    loudness.value = String(config.loudnessLufs);
    peak.value = String(config.truePeakDb);
    paragraphPause.value = String(config.paragraphPauseMs);
    chapterPause.value = String(config.chapterPauseMs);
    wavPrecision.value = config.wavPrecision;
    metadata.value = structuredClone(config.metadata);
  }
  function value() {
    return {
      format: format.value,
      sampleRate: Number(sampleRate.value),
      loudnessLufs: Number(loudness.value),
      truePeakDb: Number(peak.value),
      paragraphPauseMs: Number(paragraphPause.value),
      chapterPauseMs: Number(chapterPause.value),
      wavPrecision: wavPrecision.value,
      metadata: metadata.value,
    };
  }
  function valid() {
    const config = value();
    return (
      !(config.format === "wav" && config.metadata.cover !== null) &&
      [config.paragraphPauseMs, config.chapterPauseMs].every(Number.isSafeInteger) &&
      config.paragraphPauseMs >= 0 &&
      config.paragraphPauseMs <= 10000 &&
      config.chapterPauseMs >= 0 &&
      config.chapterPauseMs <= 30000 &&
      [config.metadata.title, config.metadata.author, config.metadata.narrator].every(
        (text) => [...text].length <= 200,
      )
    );
  }
  return {
    format,
    sampleRate,
    loudness,
    peak,
    paragraphPause,
    chapterPause,
    wavPrecision,
    metadata,
    load,
    value,
    valid,
  };
}
