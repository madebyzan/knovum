type PauseCallback = () => void;

let activePause: PauseCallback | undefined;

/**
 * Ensures only one canvas or reader media player plays audio at a time.
 * Calling `claimActiveMediaPlayback` pauses any previously active player.
 */
export function claimActiveMediaPlayback(pause: PauseCallback) {
  if (activePause && activePause !== pause) {
    const previous = activePause;
    activePause = pause;
    previous();
    return;
  }
  activePause = pause;
}

export function releaseActiveMediaPlayback(pause: PauseCallback) {
  if (activePause === pause) activePause = undefined;
}
