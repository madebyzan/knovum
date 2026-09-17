import { convertFileSrc } from "@tauri-apps/api/core";
import { withQuery } from "ufo";

export function versionedMediaSource(path?: string, version?: string | number) {
  if (!path) return undefined;
  const source = convertFileSrc(path);
  return version ? withQuery(source, { v: String(version) }) : source;
}

export function waitForMediaMetadata(element: HTMLMediaElement, failureMessage: string) {
  if (element.readyState >= HTMLMediaElement.HAVE_METADATA) return Promise.resolve();
  return new Promise<void>((resolve, reject) => {
    const cleanup = () => {
      element.removeEventListener("loadedmetadata", onLoaded);
      element.removeEventListener("error", onFailed);
    };
    const onLoaded = () => {
      cleanup();
      resolve();
    };
    const onFailed = () => {
      cleanup();
      reject(new Error(failureMessage));
    };
    element.addEventListener("loadedmetadata", onLoaded, { once: true });
    element.addEventListener("error", onFailed, { once: true });
  });
}
