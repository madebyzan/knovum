/* oxlint-disable unicorn/require-post-message-target-origin -- DedicatedWorkerGlobalScope.postMessage has no targetOrigin parameter. */
const wordSegmenter = new Intl.Segmenter(undefined, { granularity: "word" });

globalThis.addEventListener("message", (event: MessageEvent<{ revision: number; text: string }>) => {
  let wordCount = 0;
  for (const segment of wordSegmenter.segment(event.data.text)) {
    if (segment.isWordLike) wordCount += 1;
  }
  globalThis.postMessage({
    revision: event.data.revision,
    wordCount,
    characterCount: event.data.text.length,
  });
});
