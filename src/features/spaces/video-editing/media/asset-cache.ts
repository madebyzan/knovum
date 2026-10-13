import type { VideoAsset } from "../project";

/** One identity per asset revision: a new revision is different media, never a refreshed copy. */
export function assetIdentity(asset: VideoAsset) {
  return `${asset.nodeId}\u0000${asset.sourceRevision}`;
}

/**
 * Opens a piece of media once per asset revision. Concurrent requests share the same work, and a
 * failed open is not remembered, so the next request retries instead of inheriting the failure.
 */
export function createAssetCache<T>(open: (asset: VideoAsset) => Promise<T>) {
  const ready = new Map<string, T>();
  const pending = new Map<string, Promise<T>>();

  return {
    ready: (asset: VideoAsset) => ready.get(assetIdentity(asset)),
    /** Whether the asset has been opened, including media that turned out to carry nothing. */
    loaded: (asset: VideoAsset) => ready.has(assetIdentity(asset)),
    load(asset: VideoAsset) {
      const key = assetIdentity(asset);
      if (ready.has(key)) return Promise.resolve(ready.get(key)!);
      const inFlight = pending.get(key);
      if (inFlight) return inFlight;
      const request = open(asset)
        .then((value) => {
          ready.set(key, value);
          return value;
        })
        .finally(() => pending.delete(key));
      pending.set(key, request);
      return request;
    },
    forget(asset: VideoAsset) {
      const key = assetIdentity(asset);
      const value = ready.get(key);
      ready.delete(key);
      return value;
    },
    opened: () => [...ready.values()],
    clear() {
      ready.clear();
      pending.clear();
    },
  };
}
