export class FakeAudioBuffer {
  samples: Float32Array | undefined;
  copyToChannel(samples: Float32Array) {
    this.samples = samples;
  }
}

export class FakeAudioSource {
  buffer: AudioBuffer | null = null;
  startedAt: number | undefined;
  stopped = false;
  private ended: EventListener | undefined;
  connect() {}
  disconnect() {}
  start(time?: number) {
    this.startedAt = time;
  }
  stop() {
    this.stopped = true;
  }
  addEventListener(_type: string, listener: EventListenerOrEventListenerObject) {
    this.ended = typeof listener === "function" ? listener : (event) => listener.handleEvent(event);
  }
  removeEventListener(_type: string, listener: EventListenerOrEventListenerObject) {
    if (this.ended === listener) this.ended = undefined;
  }
  finish() {
    this.ended?.(new Event("ended"));
  }
}

export class FakeAnalyser {
  fftSize = 2048;
  amplitude = 0;
  connect() {}
  disconnect() {}
  getByteTimeDomainData(target: Uint8Array) {
    target.fill(128 + Math.round(this.amplitude * 127));
  }
}

export class FakeAudioContext {
  static instances: FakeAudioContext[] = [];
  static resumeGates: Promise<void>[] = [];
  state: AudioContextState = "suspended";
  currentTime = 0;
  readonly destination = {} as AudioDestinationNode;
  readonly sources: FakeAudioSource[] = [];
  readonly analysers: FakeAnalyser[] = [];
  readonly buffers: FakeAudioBuffer[] = [];
  private readonly resumeGate: Promise<void> | undefined;

  constructor() {
    this.resumeGate = FakeAudioContext.resumeGates.shift();
    FakeAudioContext.instances.push(this);
  }
  async resume() {
    await this.resumeGate;
    if (this.state !== "closed") this.state = "running";
  }
  async close() {
    this.state = "closed";
  }
  createBuffer() {
    const buffer = new FakeAudioBuffer();
    this.buffers.push(buffer);
    return buffer as unknown as AudioBuffer;
  }
  createBufferSource() {
    const source = new FakeAudioSource();
    this.sources.push(source);
    return source as unknown as AudioBufferSourceNode;
  }
  createAnalyser() {
    const analyser = new FakeAnalyser();
    this.analysers.push(analyser);
    return analyser as unknown as AnalyserNode;
  }
}
