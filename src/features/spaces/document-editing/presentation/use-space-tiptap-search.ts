import type { Editor } from "@tiptap/vue-3";
import { computed, nextTick, onBeforeUnmount, ref, shallowRef, watch, type ShallowRef } from "vue";

interface SearchInputs {
  readonly search: Readonly<ShallowRef<HTMLInputElement | null>>;
  readonly replace: Readonly<ShallowRef<HTMLInputElement | null>>;
}

interface FindAndReplaceStorage {
  readonly results: unknown[];
  readonly currentIndex: number | null;
}

export function useSpaceTiptapSearch(getEditor: () => Editor, inputs: SearchInputs) {
  const open = ref(false);
  const showReplace = ref(false);
  const query = ref("");
  const replaceTerm = ref("");
  const matchTotal = shallowRef(0);
  const matchCurrent = shallowRef(0);

  const matchSummary = computed(() => {
    if (!query.value.trim()) return "";
    return matchTotal.value === 0 ? "0/0" : `${matchCurrent.value}/${matchTotal.value}`;
  });

  function syncMatches() {
    const editor = getEditor();
    if (editor.isDestroyed) return;
    const storage = (editor.storage as { findAndReplace?: FindAndReplaceStorage })?.findAndReplace;
    if (!storage) return;
    const total = storage.results.length;
    matchTotal.value = total;
    matchCurrent.value = total > 0 && storage.currentIndex !== null ? storage.currentIndex + 1 : 0;
  }

  watch(
    getEditor,
    (instance, _prev, cleanup) => {
      syncMatches();
      instance.on("transaction", syncMatches);
      cleanup(() => instance.off("transaction", syncMatches));
    },
    { immediate: true },
  );

  watch(query, (value) => {
    const editor = getEditor();
    if (value) editor.commands.setSearchTerm(value);
    else editor.commands.clearSearch();
    syncMatches();
  });

  watch(replaceTerm, (value) => {
    getEditor().commands.setReplaceTerm(value);
  });

  watch(open, async (value) => {
    const editor = getEditor();
    if (value) {
      await nextTick();
      inputs.search.value?.focus();
      inputs.search.value?.select();
      return;
    }
    query.value = "";
    replaceTerm.value = "";
    showReplace.value = false;
    matchTotal.value = 0;
    matchCurrent.value = 0;
    editor.commands.clearSearch();
    editor.commands.focus();
  });

  function move(direction: "next" | "previous") {
    if (!query.value) return;
    const editor = getEditor();
    if (direction === "next") editor.commands.goToNextResult();
    else editor.commands.goToPreviousResult();
    syncMatches();
  }

  function handleSearchKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      open.value = false;
      return;
    }
    if (event.key === "Enter" || ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "g")) {
      event.preventDefault();
      move(event.shiftKey ? "previous" : "next");
    }
  }

  function toggleReplace() {
    showReplace.value = !showReplace.value;
    if (showReplace.value) void nextTick(() => inputs.replace.value?.focus());
  }

  function replaceMatch() {
    if (!query.value) return;
    getEditor().commands.replace();
    move("next");
  }

  function replaceAllMatches() {
    if (!query.value) return;
    getEditor().commands.replaceAll();
    syncMatches();
  }

  onBeforeUnmount(() => {
    const editor = getEditor();
    if (open.value && !editor.isDestroyed) editor.commands.clearSearch();
  });

  return {
    open,
    showReplace,
    query,
    replaceTerm,
    matchTotal,
    matchSummary,
    show: () => {
      open.value = true;
    },
    toggleReplace,
    move,
    handleSearchKeydown,
    replaceMatch,
    replaceAllMatches,
  };
}
