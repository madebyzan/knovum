let shared: AudioContext | undefined;

/**
 * The editor's single audio context, created on first use. Browsers cap the number of contexts, and
 * nothing should open audio hardware until a preview actually needs to sound.
 */
export function audioContext() {
  shared ??= new AudioContext();
  return shared;
}
