/* oxlint-disable unicorn/require-post-message-target-origin -- DedicatedWorkerGlobalScope.postMessage has no targetOrigin parameter. */
import { tokenizeCodeLines } from "./code-highlighter-engine";

interface HighlightRequest {
  id: number;
  code: string;
  language: string;
  dark: boolean;
}

async function handleHighlight({ id, code, language, dark }: HighlightRequest) {
  try {
    const lines = await tokenizeCodeLines(code, language, dark);
    globalThis.postMessage({ id, lines });
  } catch {
    globalThis.postMessage({ id, lines: undefined });
  }
}

globalThis.addEventListener("message", (event: MessageEvent<HighlightRequest>) => {
  void handleHighlight(event.data);
});
