import type { SpeechPreviewAudioChunk } from "../contracts/generated/SpeechPreviewAudioChunk";

const MAX_QUEUED_BYTES = 16 * 1024 * 1024;
const MAX_AUDIO_AHEAD_SECONDS = 120;
const START_DELAY_SECONDS = 0.03;

function isPcmByte(value: number) {
  return Number.isInteger(value) && value >= 0 && value <= 255;
}

function decodePcm16(bytes: number[], sampleCount: number) {
  if (!bytes.every(isPcmByte)) throw new Error("Preview audio chunk contains invalid PCM bytes");
  const data = Uint8Array.from(bytes);
  const view = new DataView(data.buffer);
  return Float32Array.from({ length: sampleCount }, (_, index) => view.getInt16(index * 2, true) / 32_768);
}

export class PreviewAudioQueue {
  private context: AudioContext | undefined;
  private requestId: string | undefined;
  private expectedSeq = 1;
  private sampleRate: number | undefined;
  private scheduledUntil = 0;
  private queuedBytes = 0;
  private readonly sources = new Set<AudioBufferSourceNode>();
  private readonly completionListeners = new Map<AudioBufferSourceNode, EventListener>();
  private epoch = 0;

  constructor(private readonly setPlaying: (playing: boolean) => void) {}

  async start(requestId: string) {
    this.stop();
    const epoch = this.epoch;
    const context = new AudioContext({ latencyHint: "interactive" });
    this.context = context;
    this.requestId = requestId;
    this.scheduledUntil = context.currentTime;
    await context.resume();
    if (epoch !== this.epoch || this.context !== context) {
      if (context.state !== "closed") await context.close();
      return false;
    }
    return true;
  }

  enqueue(chunk: SpeechPreviewAudioChunk) {
    const context = this.context;
    if (!this.accepts(context, chunk)) return;
    this.validateMetadata(chunk);
    const duration = chunk.sampleCount / chunk.sampleRate;
    const ahead = Math.max(0, this.scheduledUntil - context.currentTime);
    const capacityExceeded = [
      this.queuedBytes + chunk.pcm16.length > MAX_QUEUED_BYTES,
      ahead + duration > MAX_AUDIO_AHEAD_SECONDS,
    ].some(Boolean);
    if (capacityExceeded) throw new Error("Preview audio queue exceeded its playback window");
    const samples = decodePcm16(chunk.pcm16, chunk.sampleCount);
    const buffer = context.createBuffer(1, samples.length, chunk.sampleRate);
    buffer.copyToChannel(samples, 0);
    const source = context.createBufferSource();
    source.buffer = buffer;
    source.connect(context.destination);
    const byteLength = chunk.pcm16.length;
    const startAt = Math.max(context.currentTime + START_DELAY_SECONDS, this.scheduledUntil);
    this.scheduledUntil = startAt + duration;
    this.queuedBytes += byteLength;
    this.expectedSeq += 1;
    this.sampleRate = chunk.sampleRate;
    this.sources.add(source);
    this.setPlaying(true);
    const completed = () => {
      this.sources.delete(source);
      this.completionListeners.delete(source);
      this.queuedBytes = Math.max(0, this.queuedBytes - byteLength);
      source.disconnect();
      if (this.sources.size === 0) this.setPlaying(false);
    };
    this.completionListeners.set(source, completed);
    source.addEventListener("ended", completed, { once: true });
    source.start(startAt);
  }

  stop(requestId?: string) {
    if (requestId !== undefined && requestId !== this.requestId) return;
    this.epoch += 1;
    for (const source of this.sources) {
      const completed = this.completionListeners.get(source);
      if (completed) source.removeEventListener("ended", completed);
      try {
        source.stop();
      } catch {
        // The source already reached its natural end.
      }
      source.disconnect();
    }
    this.sources.clear();
    this.completionListeners.clear();
    const context = this.context;
    this.context = undefined;
    this.requestId = undefined;
    this.expectedSeq = 1;
    this.sampleRate = undefined;
    this.scheduledUntil = 0;
    this.queuedBytes = 0;
    this.setPlaying(false);
    if (context && context.state !== "closed") void context.close();
  }

  private accepts(
    context: AudioContext | undefined,
    chunk: SpeechPreviewAudioChunk,
  ): context is AudioContext {
    return Boolean(
      context && context.state !== "closed" && chunk.requestId === this.requestId && chunk.attemptEpoch === 0,
    );
  }

  private validateMetadata(chunk: SpeechPreviewAudioChunk) {
    const invalid = [
      chunk.seq !== this.expectedSeq,
      !Number.isSafeInteger(chunk.sampleRate),
      chunk.sampleRate <= 0,
      !Number.isSafeInteger(chunk.sampleCount),
      chunk.sampleCount <= 0,
      this.sampleRate !== undefined && chunk.sampleRate !== this.sampleRate,
      chunk.pcm16.length === 0,
      chunk.pcm16.length % 2 !== 0,
      chunk.sampleCount !== chunk.pcm16.length / 2,
    ].some(Boolean);
    if (invalid) throw new Error("Preview audio chunk contract is invalid");
  }
}
