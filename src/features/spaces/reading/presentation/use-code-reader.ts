import { convertFileSrc } from "@tauri-apps/api/core";
import { watchDebounced } from "@vueuse/core";
import { computed, onBeforeUnmount, ref, shallowRef, watch, type ComputedRef, type Ref } from "vue";

import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceCodePayload } from "../../model/nodes/node-payload";
import { highlightCodeLines, normalizeCodeLanguage } from "../code-highlighter";

const MAX_SHIKI_BYTES = 200_000;
const MAX_SHIKI_LINES = 3_000;

/**
 * Owns abortable code-asset loading, off-main-thread cached Shiki highlighting (preserving
 * highlighted lines across preview/code mode toggles), and debounced line matching.
 */
export function useCodeReader(options: {
  node: ComputedRef<SpaceCanvasNode<SpaceCodePayload>>;
  chunks: ComputedRef<readonly KnowledgeChunk[]>;
  isDark: Ref<boolean>;
  viewMode: Readonly<Ref<"code" | "preview">>;
}) {
  const loadedText = shallowRef<string>();
  const loadingAsset = shallowRef(false);
  const highlightedLines = shallowRef<readonly string[]>();
  const findOpen = shallowRef(false);
  const findQuery = ref("");
  const matchCursor = shallowRef(0);
  const matchedLineNumbers = shallowRef<readonly number[]>([]);
  let fetchController: AbortController | undefined;
  let loadToken = 0;
  let highlightToken = 0;
  let highlightedText = "";
  let highlightedLang: string | undefined;
  let highlightedDark: boolean | undefined;

  const language = computed(() =>
    normalizeCodeLanguage(options.node.value.data.previewLanguage, options.node.value.data.fileName),
  );
  const fallbackText = computed(() => {
    const codeChunks = options.chunks.value.filter((chunk) => chunk.kind !== "node");
    return codeChunks.length
      ? codeChunks.map((chunk) => chunk.text).join("\n")
      : (options.node.value.data.previewText ?? "");
  });
  const sourceText = computed(() => loadedText.value ?? fallbackText.value);
  const rawLines = computed(() => sourceText.value.split(/\r?\n/u));
  const activeMatchLine = computed(() => matchedLineNumbers.value[matchCursor.value]);

  async function ensureLoaded(): Promise<string> {
    if (loadedText.value !== undefined) return loadedText.value;
    const assetPath = options.node.value.data.assetPath;
    if (!assetPath) return fallbackText.value;
    fetchController?.abort();
    const controller = new AbortController();
    fetchController = controller;
    const token = ++loadToken;
    loadingAsset.value = true;
    try {
      const response = await fetch(convertFileSrc(assetPath), { signal: controller.signal });
      if (!response.ok) return sourceText.value;
      const text = await response.text();
      if (token === loadToken) loadedText.value = text;
      return text;
    } catch {
      return sourceText.value;
    } finally {
      if (token === loadToken) loadingAsset.value = false;
    }
  }

  function recomputeMatches() {
    const needle = findQuery.value.trim().toLowerCase();
    if (!needle) {
      matchedLineNumbers.value = [];
      matchCursor.value = 0;
      return;
    }
    const hits: number[] = [];
    for (const [idx, line] of rawLines.value.entries()) {
      if (line.toLowerCase().includes(needle)) hits.push(idx + 1);
    }
    matchedLineNumbers.value = hits;
    matchCursor.value = 0;
  }

  function resetFind() {
    findOpen.value = false;
    findQuery.value = "";
    matchedLineNumbers.value = [];
    matchCursor.value = 0;
  }

  function clearHighlight() {
    highlightedLines.value = undefined;
    highlightedText = "";
  }

  function isHighlightCurrent(text: string, lang: string | undefined, dark: boolean): boolean {
    return Boolean(
      highlightedLines.value &&
      text === highlightedText &&
      lang === highlightedLang &&
      dark === highlightedDark,
    );
  }

  async function refreshHighlight(text: string, lang: string | undefined, dark: boolean) {
    if (isHighlightCurrent(text, lang, dark)) return;
    const token = ++highlightToken;
    if (!text || text.length > MAX_SHIKI_BYTES || rawLines.value.length > MAX_SHIKI_LINES) {
      clearHighlight();
      return;
    }
    try {
      const lines = await highlightCodeLines(text, lang, dark);
      if (token !== highlightToken) return;
      highlightedLines.value = lines;
      highlightedText = lines ? text : "";
      highlightedLang = lang;
      highlightedDark = dark;
    } catch {
      if (token === highlightToken) clearHighlight();
    }
  }

  watch(
    () => [options.node.value.data.assetPath, options.viewMode.value] as const,
    ([assetPath, mode], prev) => {
      if (assetPath !== prev?.[0]) {
        fetchController?.abort();
        loadedText.value = undefined;
        clearHighlight();
      }
      if (mode === "code" && assetPath && loadedText.value === undefined) void ensureLoaded();
    },
    { immediate: true },
  );

  watch(
    [sourceText, language, options.isDark, options.viewMode, loadingAsset],
    ([text, lang, dark, mode, isFetching]) => {
      if (mode === "code" && !isFetching) void refreshHighlight(text, lang, dark);
    },
    { immediate: true },
  );

  watchDebounced(findQuery, recomputeMatches, { debounce: 80 });
  onBeforeUnmount(() => fetchController?.abort());

  return {
    language,
    sourceText,
    rawLines,
    highlightedLines,
    findOpen,
    findQuery,
    matchCursor,
    matchedLineNumbers,
    activeMatchLine,
    ensureLoaded,
    resetFind,
  };
}
