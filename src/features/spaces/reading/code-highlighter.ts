import { tokenizeCodeLines } from "./code-highlighter-engine";

const LANGUAGE_ALIASES: Record<string, string> = {
  js: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  ts: "typescript",
  mts: "typescript",
  cts: "typescript",
  py: "python",
  rs: "rust",
  rb: "ruby",
  sh: "shellscript",
  bash: "shellscript",
  zsh: "shellscript",
  yml: "yaml",
  md: "markdown",
  kt: "kotlin",
  cs: "csharp",
  "c++": "cpp",
  "c#": "csharp",
  dockerfile: "docker",
  plaintext: "text",
  txt: "text",
};

const MAX_CACHE_ENTRIES = 12;
const highlightCache = new Map<string, readonly string[]>();

let workerInstance: Worker | undefined;
let workerFailed = false;
let nextRequestId = 0;
const pendingRequests = new Map<number, (lines: readonly string[] | undefined) => void>();

function cacheGet(key: string): readonly string[] | undefined {
  const cached = highlightCache.get(key);
  if (!cached) return undefined;
  highlightCache.delete(key);
  highlightCache.set(key, cached);
  return cached;
}

function cacheSet(key: string, lines: readonly string[]) {
  highlightCache.delete(key);
  highlightCache.set(key, lines);
  if (highlightCache.size > MAX_CACHE_ENTRIES) {
    const oldest = highlightCache.keys().next().value;
    if (oldest !== undefined) highlightCache.delete(oldest);
  }
}

function getWorker(): Worker | undefined {
  if (workerFailed || typeof Worker === "undefined") return undefined;
  if (workerInstance) return workerInstance;
  try {
    const worker = new Worker(new URL("./code-highlighter-worker.ts", import.meta.url), {
      type: "module",
    });
    worker.addEventListener(
      "message",
      (event: MessageEvent<{ id: number; lines?: readonly string[] | undefined }>) => {
        const resolve = pendingRequests.get(event.data.id);
        if (!resolve) return;
        pendingRequests.delete(event.data.id);
        resolve(event.data.lines);
      },
    );
    worker.addEventListener("error", () => {
      workerFailed = true;
      workerInstance = undefined;
      for (const resolve of pendingRequests.values()) resolve(undefined);
      pendingRequests.clear();
    });
    workerInstance = worker;
    return worker;
  } catch {
    workerFailed = true;
    return undefined;
  }
}

function runHighlight(code: string, language: string, dark: boolean): Promise<readonly string[] | undefined> {
  const worker = getWorker();
  if (!worker) return tokenizeCodeLines(code, language, dark);
  return new Promise((resolve) => {
    const id = ++nextRequestId;
    pendingRequests.set(id, resolve);
    // oxlint-disable-next-line unicorn/require-post-message-target-origin -- Worker.postMessage has no targetOrigin parameter.
    worker.postMessage({ id, code, language, dark });
  });
}

export function normalizeCodeLanguage(raw?: string, fileName?: string): string | undefined {
  const fromRaw = raw
    ?.trim()
    .toLowerCase()
    .replace(/^language-/u, "");
  const fromFile = fileName?.includes(".") ? fileName.split(".").at(-1)?.toLowerCase() : undefined;
  const candidate = fromRaw || fromFile;
  if (!candidate) return undefined;
  return LANGUAGE_ALIASES[candidate] ?? candidate;
}

/**
 * Highlights source code into per-line HTML strings off the UI main thread via a dedicated Shiki
 * Web Worker with an LRU cache, so large files never block view switching or scrolling.
 */
export async function highlightCodeLines(
  code: string,
  language: string | undefined,
  dark: boolean,
): Promise<readonly string[] | undefined> {
  const normalized = normalizeCodeLanguage(language);
  if (!normalized || normalized === "text") return undefined;
  const key = `${dark ? 1 : 0}:${normalized}\0${code}`;
  const cached = cacheGet(key);
  if (cached) return cached;
  const lines = await runHighlight(code, normalized, dark);
  if (lines) cacheSet(key, lines);
  return lines;
}

function languageFromCodeElement(codeElement: HTMLElement): string | undefined {
  for (const className of codeElement.classList) {
    if (className.startsWith("language-")) return normalizeCodeLanguage(className);
  }
  return undefined;
}

/** Enhances fenced code blocks inside rendered Markdown or Note containers in-place via Shiki. */
export async function highlightContainerCodeBlocks(container: HTMLElement, dark: boolean): Promise<void> {
  const codeBlocks = [...container.querySelectorAll<HTMLElement>("pre > code")];
  const themeKey = dark ? "dark" : "light";
  await Promise.all(
    codeBlocks.map(async (codeElement) => {
      if (codeElement.dataset.shikiTheme === themeKey) return;
      const language = languageFromCodeElement(codeElement);
      if (!language) return;
      const rawSource = codeElement.dataset.rawCode ?? codeElement.textContent ?? "";
      codeElement.dataset.rawCode = rawSource;
      const lines = await highlightCodeLines(rawSource, language, dark);
      if (!lines || !codeElement.isConnected) return;
      codeElement.innerHTML = lines.join("\n");
      codeElement.dataset.shikiTheme = themeKey;
    }),
  );
}
