/**
 * Reduces a decoded buffer to one absolute peak per bucket. The waveform draws these directly, so
 * the timeline never walks the samples again while the user scrolls or zooms.
 */
export function computePeaks(buffer: AudioBuffer, buckets: number) {
  const size = Math.max(1, buckets);
  const peaks = new Float32Array(size);
  const samplesPerBucket = Math.max(1, Math.floor(buffer.length / size));
  for (let channel = 0; channel < buffer.numberOfChannels; channel += 1) {
    const samples = buffer.getChannelData(channel);
    for (let bucket = 0; bucket < size; bucket += 1) {
      const start = bucket * samplesPerBucket;
      const end = Math.min(samples.length, start + samplesPerBucket);
      let peak = peaks[bucket]!;
      for (let index = start; index < end; index += 1) {
        const value = Math.abs(samples[index]!);
        if (value > peak) peak = value;
      }
      peaks[bucket] = peak;
    }
  }
  return peaks;
}
