import type { SpeechPreviewAudioChunk } from "../contracts/generated/SpeechPreviewAudioChunk";

const MAX_QUEUED_BYTES = 16 * 1024 * 1024;
const MAX_AUDIO_AHEAD_SECONDS = 120;
const START_DELAY_SECONDS = 0.03;

const isPcmByte = (value: number) => Number.isInteger(value) && value >= 0 && value <= 255;
const isValidPcm = (pcm: number[], count: number) => pcm.length > 0 && pcm.length === count * 2;
const isValidRate = (rate: number, exp?: number) =>
  Number.isSafeInteger(rate) && rate > 0 && (exp === undefined || rate === exp);

function decodePcm16(bytes: number[], sampleCount: number) {
  if (!bytes.every(isPcmByte)) throw new Error("Preview audio chunk contains invalid PCM bytes");
  const view = new DataView(Uint8Array.from(bytes).buffer);
  return Float32Array.from({ length: sampleCount }, (_, i) => view.getInt16(i * 2, true) / 32_768);
}

export class PreviewAudioQueue {
  private context: AudioContext | undefined;
  private analyser: AnalyserNode | undefined;
  private levels: Uint8Array<ArrayBuffer> | undefined;
  private requestId: string | undefined;
  private expectedSeq = 1;
  private sampleRate: number | undefined;
  private scheduledUntilSeconds = 0;
  private queuedBytes = 0;
  private readonly sources = new Set<AudioBufferSourceNode>();
  private readonly completionListenerBySource = new Map<AudioBufferSourceNode, EventListener>();
  private pending: SpeechPreviewAudioChunk[] = [];
  private pendingSeconds = 0;
  private epoch = 0;
  private readonly endedListeners = new Set<() => void>();

  constructor(private readonly setPlaying: (playing: boolean) => void) {}

  async start(requestId: string) {
    this.stop();
    const epoch = this.epoch;
    const context = new AudioContext({ latencyHint: "interactive" });
    this.context = context;
    this.requestId = requestId;
    this.scheduledUntilSeconds = context.currentTime;
    // Waveform draws audio being heard at this instant through the analyser.
    const analyser = context.createAnalyser();
    analyser.fftSize = 2048;
    analyser.connect(context.destination);
    this.analyser = analyser;
    this.levels = new Uint8Array(new ArrayBuffer(analyser.fftSize));
    await context.resume();
    const stale = epoch !== this.epoch || this.context !== context;
    if (stale && context.state !== "closed") await context.close();
    return !stale;
  }

  /** The loudest sample currently audible, from 0 to 1. Silence while nothing plays. */
  peak(): number {
    const analyser = this.analyser;
    const levels = this.levels;
    if (!analyser || !levels || this.sources.size === 0) return 0;
    analyser.getByteTimeDomainData(levels);
    let peak = 0;
    for (const level of levels) peak = Math.max(peak, Math.abs(level - 128) / 128);
    return peak;
  }

  enqueue(chunk: SpeechPreviewAudioChunk) {
    const context = this.context;
    if (!this.accepts(context, chunk)) return;
    this.validateMetadata(chunk);
    this.expectedSeq += 1;
    this.sampleRate = chunk.sampleRate;
    const duration = chunk.sampleCount / chunk.sampleRate;
    this.pending.push(chunk);
    this.pendingSeconds += duration;
  }

  flush() {
    const context = this.context;
    if (!context || context.state === "closed" || !this.pending.length) return;
    const queued = this.pending;
    this.pending = [];
    this.pendingSeconds = 0;
    this.scheduledUntilSeconds = Math.max(context.currentTime + START_DELAY_SECONDS, this.scheduledUntilSeconds);
    for (const chunk of queued) this.scheduleChunk(context, chunk);
  }

  waitUntilEnded(): Promise<void> {
    if (this.sources.size === 0 && this.pending.length === 0) return Promise.resolve();
    return new Promise<void>((resolve) => this.endedListeners.add(resolve));
  }

  private notifyEnded() {
    this.endedListeners.forEach((listener) => listener());
    this.endedListeners.clear();
  }

  private scheduleChunk(context: AudioContext, chunk: SpeechPreviewAudioChunk) {
    const durationSeconds = chunk.sampleCount / chunk.sampleRate;
    const aheadSeconds = Math.max(0, this.scheduledUntilSeconds - context.currentTime);
    if (
      this.queuedBytes + chunk.pcm16.length > MAX_QUEUED_BYTES ||
      aheadSeconds + durationSeconds > MAX_AUDIO_AHEAD_SECONDS
    )
      throw new Error("Preview audio queue exceeded its playback window");
    const samples = decodePcm16(chunk.pcm16, chunk.sampleCount);
    const buffer = context.createBuffer(1, samples.length, chunk.sampleRate);
    buffer.copyToChannel(samples, 0);
    const source = context.createBufferSource();
    source.buffer = buffer;
    source.connect(this.analyser ?? context.destination);
    const byteLength = chunk.pcm16.length;
    const startAt = Math.max(context.currentTime + START_DELAY_SECONDS, this.scheduledUntilSeconds);
    this.scheduledUntilSeconds = startAt + durationSeconds;
    this.queuedBytes += byteLength;
    this.sources.add(source);
    this.setPlaying(true);
    const completed = () => {
      this.sources.delete(source);
      this.completionListenerBySource.delete(source);
      this.queuedBytes = Math.max(0, this.queuedBytes - byteLength);
      source.disconnect();
      if (this.sources.size === 0) {
        this.setPlaying(false);
        this.notifyEnded();
      }
    };
    this.completionListenerBySource.set(source, completed);
    source.addEventListener("ended", completed, { once: true });
    source.start(startAt);
  }

  stop(requestId?: string) {
    if (requestId !== undefined && requestId !== this.requestId) return;
    this.epoch += 1;
    this.pending = [];
    this.pendingSeconds = 0;
    for (const source of this.sources) {
      const completed = this.completionListenerBySource.get(source);
      if (completed) source.removeEventListener("ended", completed);
      try {
        source.stop();
      } catch {
        // The source already reached its natural end.
      }
      source.disconnect();
    }
    this.sources.clear();
    this.completionListenerBySource.clear();
    this.analyser?.disconnect();
    this.analyser = undefined;
    this.levels = undefined;
    const context = this.context;
    this.context = undefined;
    this.requestId = undefined;
    this.expectedSeq = 1;
    this.sampleRate = undefined;
    this.scheduledUntilSeconds = 0;
    this.queuedBytes = 0;
    this.setPlaying(false);
    this.notifyEnded();
    if (context && context.state !== "closed") void context.close();
  }

  private accepts(context: AudioContext | undefined, chunk: SpeechPreviewAudioChunk): context is AudioContext {
    return Boolean(
      context && context.state !== "closed" && chunk.requestId === this.requestId && chunk.attemptEpoch === 0,
    );
  }

  private validateMetadata(chunk: SpeechPreviewAudioChunk) {
    const valid =
      chunk.seq === this.expectedSeq &&
      Number.isSafeInteger(chunk.sampleCount) &&
      chunk.sampleCount > 0 &&
      isValidRate(chunk.sampleRate, this.sampleRate) &&
      isValidPcm(chunk.pcm16, chunk.sampleCount);
    if (!valid) throw new Error("Preview audio chunk contract is invalid");
  }
}
