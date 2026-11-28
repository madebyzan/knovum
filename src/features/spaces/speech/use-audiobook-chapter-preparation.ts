import { computed, shallowRef } from "vue";

import type { NarrationPreparationMode } from "../../../shared/contracts/generated/NarrationPreparationMode";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import type { ChapterDraft } from "./audiobook-draft";
import { useNarrationScriptPreparation } from "./use-narration-script-preparation";

interface ChapterSnapshot {
  chapterKey: string;
  paragraphs: ChapterDraft["paragraphs"];
}

interface BookRestoreSnapshot {
  scope: "chapter" | "book";
  chapterKey?: string | undefined;
  chapters: ChapterSnapshot[];
}

function chapterSourceText(chapter: ChapterDraft) {
  return chapter.paragraphs
    .map((paragraph) => paragraph.text.trim())
    .filter(Boolean)
    .join("\n\n");
}

function snapshotChapter(chapter: ChapterDraft): ChapterSnapshot {
  return {
    chapterKey: chapter.key,
    paragraphs: chapter.paragraphs.map((paragraph) => ({ ...paragraph })),
  };
}

function applyCleanedBlocks(chapter: ChapterDraft, cleaned: string, defaultRole: string) {
  const blocks = cleaned
    .split(/\n{2,}/u)
    .map((block) => block.trim())
    .filter(Boolean);
  if (!blocks.length) return;
  chapter.paragraphs = blocks.map((block, index) => ({
    key: chapter.paragraphs[index]?.key ?? crypto.randomUUID(),
    speakerKey: chapter.paragraphs[index]?.speakerKey ?? defaultRole,
    text: block,
    deliveryInstruction: chapter.paragraphs[index]?.deliveryInstruction ?? "",
  }));
}

export function useAudiobookChapterPreparation(defaultRoleKey: () => string) {
  const preparation = useNarrationScriptPreparation<ChapterSnapshot>();
  const batchRunning = shallowRef(false);
  const preparingScope = shallowRef<"chapter" | "book">();
  const preparingChapterKey = shallowRef<string>();
  const batchProgress = shallowRef<{ current: number; total: number }>();
  const restoreSnapshot = shallowRef<BookRestoreSnapshot>();
  let batchCancelled = false;

  const preparing = computed(() => batchRunning.value || preparation.preparing.value);
  const canRestoreBook = computed(() => !preparing.value && restoreSnapshot.value?.scope === "book");

  async function runSingleChapter(
    chapter: ChapterDraft,
    mode: NarrationPreparationMode,
    target?: TaskTarget,
  ) {
    const sourceText = chapterSourceText(chapter);
    if (!sourceText) return false;
    const defaultRole = chapter.paragraphs[0]?.speakerKey ?? defaultRoleKey();
    const snapshot = snapshotChapter(chapter);
    preparingChapterKey.value = chapter.key;
    return await preparation
      .run({
        mode,
        sourceText,
        target,
        snapshot,
        onComplete: (cleaned) => applyCleanedBlocks(chapter, cleaned, defaultRole),
        onRestore: (previous) => {
          if (chapter.key === previous.chapterKey) chapter.paragraphs = previous.paragraphs;
        },
      })
      .finally(() => {
        if (preparingChapterKey.value === chapter.key) preparingChapterKey.value = undefined;
      });
  }

  async function prepareChapter(chapter: ChapterDraft, mode: NarrationPreparationMode, target?: TaskTarget) {
    if (preparing.value || !chapterSourceText(chapter)) return;
    const snapshot = snapshotChapter(chapter);
    preparingScope.value = "chapter";
    try {
      const ok = await runSingleChapter(chapter, mode, target);
      if (ok) {
        restoreSnapshot.value = { scope: "chapter", chapterKey: chapter.key, chapters: [snapshot] };
      }
    } finally {
      if (preparingScope.value === "chapter") preparingScope.value = undefined;
    }
  }

  async function prepareAllChapters(
    chapters: ChapterDraft[],
    mode: NarrationPreparationMode,
    target?: TaskTarget,
  ) {
    const targets = chapters.filter((chapter) => chapterSourceText(chapter).length > 0);
    if (preparing.value || !targets.length) return;
    const snapshots = targets.map(snapshotChapter);
    batchCancelled = false;
    batchRunning.value = true;
    preparingScope.value = "book";
    let completed = 0;
    const runNext = async (index: number): Promise<void> => {
      if (batchCancelled || index >= targets.length) return;
      batchProgress.value = { current: index + 1, total: targets.length };
      const ok = await runSingleChapter(targets[index]!, mode, target);
      if (!ok) return;
      completed += 1;
      await runNext(index + 1);
    };
    try {
      await runNext(0);
      if (completed > 0) {
        restoreSnapshot.value = { scope: "book", chapters: snapshots };
      }
    } finally {
      batchRunning.value = false;
      preparingScope.value = undefined;
      preparingChapterKey.value = undefined;
      batchProgress.value = undefined;
    }
  }

  function restoreChapter(chapter: ChapterDraft) {
    const entry = restoreSnapshot.value?.chapters.find(({ chapterKey }) => chapterKey === chapter.key);
    if (!entry) return;
    chapter.paragraphs = entry.paragraphs;
    restoreSnapshot.value = undefined;
    preparation.clearHistory();
  }

  function restoreAllChapters(chapters: ChapterDraft[]) {
    const saved = restoreSnapshot.value;
    if (!saved) return;
    const byKey = new Map(saved.chapters.map((entry) => [entry.chapterKey, entry.paragraphs]));
    for (const chapter of chapters) {
      const paragraphs = byKey.get(chapter.key);
      if (paragraphs) chapter.paragraphs = paragraphs;
    }
    restoreSnapshot.value = undefined;
    preparation.clearHistory();
  }

  function cancel() {
    batchCancelled = true;
    preparation.cancel();
  }

  return {
    preparing,
    preparingScope,
    preparingChapterKey,
    batchProgress,
    canRestoreBook,
    canRestoreChapter: (chapterKey: string) =>
      !preparing.value &&
      restoreSnapshot.value?.scope === "chapter" &&
      restoreSnapshot.value.chapterKey === chapterKey,
    prepareChapter,
    prepareAllChapters,
    restoreChapter,
    restoreAllChapters,
    cancel,
  };
}

export type AudiobookChapterPreparation = ReturnType<typeof useAudiobookChapterPreparation>;
