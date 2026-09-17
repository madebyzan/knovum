export interface SpaceMediaSeekRequest {
  id: string;
  positionMs: number;
  nonce: number;
}

export interface SpaceMediaToggleRequest {
  id: string;
  nonce: number;
}

export interface SpaceMediaPlaybackState {
  currentMs: number;
  durationMs: number;
  playing: boolean;
}
