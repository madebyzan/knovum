import { watch, type Ref, type ShallowRef } from "vue";

import { registerEditorHistory } from "../../../../foundation/commands/editing-responder";
import type { PdfAnnotationTool, PdfReaderSession } from "./pdf-editor-params";

interface PdfShortcutOptions {
  container: Readonly<Ref<HTMLDivElement | null>>;
  session: Readonly<ShallowRef<PdfReaderSession | undefined>>;
  viewMode: Readonly<Ref<"pdf" | "text">>;
  areaMode: Ref<boolean>;
  findOpen: Ref<boolean>;
  modified: Readonly<Ref<boolean>>;
  markupOpen: Readonly<Ref<boolean>>;
  markupTool: Readonly<Ref<PdfAnnotationTool>>;
  selectMarkupTool: (tool: PdfAnnotationTool) => void;
  toggleMarkup: () => void;
  savePdf: () => Promise<void>;
}

/** Binds PDF reader shortcuts (Cmd+F/S/+/-/0, Escape) and routes macOS Undo/Redo during markup. */
export function usePdfReaderShortcuts(options: PdfShortcutOptions) {
  watch(
    [options.container, options.markupOpen],
    ([el, open], _, onCleanup) => {
      if (!el || !open) return;
      onCleanup(
        registerEditorHistory(el, {
          undo: () => options.session.value?.undo(),
          redo: () => options.session.value?.redo(),
        }),
      );
    },
    { flush: "post" },
  );

  const cmdActions: Record<string, () => void> = {
    f: () => {
      options.findOpen.value = true;
    },
    s: () => {
      if (options.modified.value) void options.savePdf();
    },
    "=": () => options.session.value?.zoom(1),
    "+": () => options.session.value?.zoom(1),
    "-": () => options.session.value?.zoom(-1),
    "0": () => options.session.value?.fitWidth(),
  };

  return function onKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      if (options.areaMode.value) options.areaMode.value = false;
      else if (options.markupOpen.value) {
        if (options.markupTool.value !== "none") options.selectMarkupTool("none");
        else options.toggleMarkup();
      }
      return;
    }
    if (!(event.metaKey || event.ctrlKey) || options.viewMode.value !== "pdf") return;
    const action = cmdActions[event.key.toLowerCase()];
    if (!action) return;
    event.preventDefault();
    action();
  };
}
