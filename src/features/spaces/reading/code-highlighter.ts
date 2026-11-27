import { tokenizeCodeLines, tokenizeCodeSpans, type CodeTokenSpan } from "./code-highlighter-engine";

export type { CodeTokenSpan };

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

const MAX_CACHE_ENTRIES = 24;
const lineCache = new Map<string, readonly string[]>();
const spanCache = new Map<string, readonly CodeTokenSpan[]>();

interface WorkerResponse {
  id: number;
  lines?: readonly string[] | undefined;
  spans?: readonly CodeTokenSpan[] | undefined;
}

let workerInstance: Worker | undefined;
let workerFailed = false;
let nextRequestId = 0;
const pendingRequests = new Map<number, (response: WorkerResponse) => void>();

function lruGet<T>(cache: Map<string, T>, key: string): T | undefined {
  const cached = cache.get(key);
  if (!cached) return undefined;
  cache.delete(key);
  cache.set(key, cached);
  return cached;
}

function lruSet<T>(cache: Map<string, T>, key: string, value: T) {
  cache.delete(key);
  cache.set(key, value);
  if (cache.size > MAX_CACHE_ENTRIES) {
    const oldest = cache.keys().next().value;
    if (oldest !== undefined) cache.delete(oldest);
  }
}

function getWorker(): Worker | undefined {
  if (workerFailed || typeof Worker === "undefined") return undefined;
  if (workerInstance) return workerInstance;
  try {
    const worker = new Worker(new URL("./code-highlighter-worker.ts", import.meta.url), {
      type: "module",
    });
    worker.addEventListener("message", (event: MessageEvent<WorkerResponse>) => {
      const resolve = pendingRequests.get(event.data.id);
      if (!resolve) return;
      pendingRequests.delete(event.data.id);
      resolve(event.data);
    });
    worker.addEventListener("error", () => {
      workerFailed = true;
      workerInstance = undefined;
      for (const resolve of pendingRequests.values()) resolve({ id: 0 });
      pendingRequests.clear();
    });
    workerInstance = worker;
    return worker;
  } catch {
    workerFailed = true;
    return undefined;
  }
}

function postWorkerRequest(
  mode: "lines" | "spans",
  code: string,
  language: string,
  dark: boolean,
): Promise<WorkerResponse | undefined> {
  const worker = getWorker();
  if (!worker) return Promise.resolve(undefined);
  return new Promise((resolve) => {
    const id = ++nextRequestId;
    pendingRequests.set(id, resolve);
    // oxlint-disable-next-line unicorn/require-post-message-target-origin -- Worker.postMessage has no targetOrigin parameter.
    worker.postMessage({ id, mode, code, language, dark });
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

/** Highlights source code into per-line HTML strings off the UI main thread via the Shiki Worker. */
export async function highlightCodeLines(
  code: string,
  language: string | undefined,
  dark: boolean,
): Promise<readonly string[] | undefined> {
  const normalized = normalizeCodeLanguage(language);
  if (!normalized || normalized === "text") return undefined;
  const key = `${dark ? 1 : 0}:${normalized}\0${code}`;
  const cached = lruGet(lineCache, key);
  if (cached) return cached;
  const response = await postWorkerRequest("lines", code, normalized, dark);
  const lines = response ? response.lines : await tokenizeCodeLines(code, normalized, dark);
  if (lines) lruSet(lineCache, key, lines);
  return lines;
}

/** Highlights source code into UTF-16 offset style spans off the UI main thread for live editors. */
export async function highlightCodeSpans(
  code: string,
  language: string | undefined,
  dark: boolean,
): Promise<readonly CodeTokenSpan[] | undefined> {
  const normalized = normalizeCodeLanguage(language);
  if (!normalized || normalized === "text" || !code) return undefined;
  const key = `${dark ? 1 : 0}:${normalized}\0${code}`;
  const cached = lruGet(spanCache, key);
  if (cached) return cached;
  const response = await postWorkerRequest("spans", code, normalized, dark);
  const spans = response ? response.spans : await tokenizeCodeSpans(code, normalized, dark);
  if (spans) lruSet(spanCache, key, spans);
  return spans;
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
      if (!language || language === "mermaid") return;
      const rawSource = codeElement.dataset.rawCode ?? codeElement.textContent ?? "";
      codeElement.dataset.rawCode = rawSource;
      const lines = await highlightCodeLines(rawSource, language, dark);
      if (!lines || !codeElement.isConnected) return;
      codeElement.innerHTML = lines.join("\n");
      codeElement.dataset.shikiTheme = themeKey;
    }),
  );
}
