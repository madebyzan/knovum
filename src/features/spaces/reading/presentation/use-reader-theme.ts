import { useMutationObserver } from "@vueuse/core";
import { shallowRef } from "vue";

function readDocumentDarkTheme(): boolean {
  return typeof document !== "undefined" && document.documentElement.getAttribute("data-theme") === "dark";
}

/** Tracks Knovum's active light/dark document theme for syntax-highlighted reading surfaces. */
export function useReaderTheme() {
  const isDark = shallowRef(readDocumentDarkTheme());
  if (typeof document !== "undefined") {
    useMutationObserver(
      () => document.documentElement,
      () => {
        isDark.value = readDocumentDarkTheme();
      },
      { attributes: true, attributeFilter: ["data-theme"] },
    );
  }
  return { isDark };
}
