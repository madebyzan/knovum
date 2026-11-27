let shared: AudioContext | undefined;

/**
 * The editor's single audio context, created on first use. Browsers cap the number of contexts, and
 * nothing should open audio hardware until a preview actually needs to sound.
 */
export function audioContext() {
  shared ??= new AudioContext();
  return shared;
}

/** Suspends the shared context when idle so the OS audio device is released between playbacks. */
export function suspendAudioContext(): void {
  if (shared?.state === "running") void shared.suspend();
}
