import { mediaSource } from "./media-manifest.mjs";

export function validateMediaDescriptor(repository, descriptor, target) {
  const expected = {
    schemaVersion: 1,
    runtimeId: "ffmpeg",
    target,
    version: mediaSource(repository, target).version,
    sourceSha256: mediaSource(repository, target).source.sha256,
    licenseReviewSha256: mediaSource(repository, target).licenseReviewSha256,
  };
  if (
    !exactKeys(descriptor, [...Object.keys(expected), "archiveUrl", "size", "sha256", "manifestSha256"]) ||
    Object.entries(expected).some(([key, value]) => descriptor[key] !== value) ||
    !validSize(descriptor.size) ||
    descriptor.size === 0 ||
    [descriptor.sha256, descriptor.manifestSha256].some((hash) => !/^[a-f0-9]{64}$/u.test(hash ?? ""))
  )
    throw new Error("Invalid core media publication descriptor");
  const url = new URL(descriptor.archiveUrl);
  if (url.protocol !== "https:" || url.username || url.password || url.hash)
    throw new Error("Invalid core media archive URL");
}

function exactKeys(value, keys) {
  return (
    value !== null &&
    typeof value === "object" &&
    Object.keys(value).toSorted().join("\0") === keys.toSorted().join("\0")
  );
}
const validSize = (size) => Number.isSafeInteger(size) && size > 0 && size <= 4 * 1024 ** 3;
