export interface SpaceUrlCaptureProjection {
  id: string;
  spaceId: string;
  url: string;
  title: string;
  position: { x: number; y: number };
  status: "preparing" | "resolving" | "downloading" | "finalizing" | "failed";
  downloadedBytes: number;
  totalBytes: number | null;
  processedMs: number | null;
  durationMs: number | null;
}
