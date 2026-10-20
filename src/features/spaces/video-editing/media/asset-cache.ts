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
  const readyByKey = new Map<string, T>();
  const pendingByKey = new Map<string, Promise<T>>();

  return {
    ready: (asset: VideoAsset) => readyByKey.get(assetIdentity(asset)),
    /** Whether the asset has been opened, including media that turned out to carry nothing. */
    loaded: (asset: VideoAsset) => readyByKey.has(assetIdentity(asset)),
    load(asset: VideoAsset) {
      const key = assetIdentity(asset);
      if (readyByKey.has(key)) return Promise.resolve(readyByKey.get(key)!);
      const inFlight = pendingByKey.get(key);
      if (inFlight) return inFlight;
      const request = open(asset)
        .then((value) => {
          readyByKey.set(key, value);
          return value;
        })
        .finally(() => pendingByKey.delete(key));
      pendingByKey.set(key, request);
      return request;
    },
    forget(asset: VideoAsset) {
      const key = assetIdentity(asset);
      const value = readyByKey.get(key);
      readyByKey.delete(key);
      return value;
    },
    opened: () => [...readyByKey.values()],
    clear() {
      readyByKey.clear();
      pendingByKey.clear();
    },
  };
}
