import { shallowRef } from "vue";

interface TextStatistics {
  revision: number;
  wordCount: number;
  characterCount: number;
}

export function createTiptapTextStatistics() {
  const wordCount = shallowRef(0);
  const characterCount = shallowRef(0);
  const worker = new Worker(new URL("./tiptap-text-statistics-worker.ts", import.meta.url), {
    type: "module",
  });
  let revision = 0;
  worker.addEventListener("message", (event: MessageEvent<TextStatistics>) => {
    if (event.data.revision !== revision) return;
    wordCount.value = event.data.wordCount;
    characterCount.value = event.data.characterCount;
  });
  const update = (text: string) => {
    revision += 1;
    // oxlint-disable-next-line unicorn/require-post-message-target-origin -- Worker.postMessage has no targetOrigin parameter.
    worker.postMessage({ revision, text });
  };
  const dispose = () => worker.terminate();
  return { wordCount, characterCount, update, dispose };
}
