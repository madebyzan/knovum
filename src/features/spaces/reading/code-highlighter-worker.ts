/* oxlint-disable unicorn/require-post-message-target-origin -- DedicatedWorkerGlobalScope.postMessage has no targetOrigin parameter. */
import { tokenizeCodeLines, tokenizeCodeSpans } from "./code-highlighter-engine";

interface HighlightRequest {
  id: number;
  mode?: "lines" | "spans";
  code: string;
  language: string;
  dark: boolean;
}

async function handleHighlight({ id, mode = "lines", code, language, dark }: HighlightRequest) {
  try {
    if (mode === "spans") {
      const spans = await tokenizeCodeSpans(code, language, dark);
      globalThis.postMessage({ id, spans });
      return;
    }
    const lines = await tokenizeCodeLines(code, language, dark);
    globalThis.postMessage({ id, lines });
  } catch {
    globalThis.postMessage(mode === "spans" ? { id, spans: undefined } : { id, lines: undefined });
  }
}

globalThis.addEventListener("message", (event: MessageEvent<HighlightRequest>) => {
  void handleHighlight(event.data);
});
