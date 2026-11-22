import { shallowRef, type ShallowRef } from "vue";

import {
  PDF_HIGHLIGHT_SWATCHES,
  PDF_INK_SWATCHES,
  type PdfAnnotationTool,
  type PdfReaderSession,
} from "./pdf-editor-params";

/** Owns PDF Markup bar tool/color/size state and forwards parameter updates to the active pdf.js session. */
export function usePdfMarkup(session: Readonly<ShallowRef<PdfReaderSession | undefined>>) {
  const open = shallowRef(false);
  const tool = shallowRef<PdfAnnotationTool>("none");
  const highlightColor = shallowRef<string>(PDF_HIGHLIGHT_SWATCHES[0]);
  const inkColor = shallowRef<string>(PDF_INK_SWATCHES[1]);
  const textSize = shallowRef(14);
  const inkWidth = shallowRef(4);

  function selectTool(next: PdfAnnotationTool) {
    tool.value = next;
    session.value?.setAnnotationTool(next);
    if (next === "highlight") session.value?.setEditorColor("highlight", highlightColor.value);
    else if (next === "freetext") {
      session.value?.setEditorColor("freetext", inkColor.value);
      session.value?.setEditorSize("freetext", textSize.value);
    } else if (next === "ink") {
      session.value?.setEditorColor("ink", inkColor.value);
      session.value?.setEditorSize("ink", inkWidth.value);
    }
  }

  function toggle() {
    open.value = !open.value;
    selectTool(open.value ? "highlight" : "none");
  }

  function reset() {
    open.value = false;
    tool.value = "none";
  }

  function setHighlightColor(hex: string) {
    highlightColor.value = hex;
    session.value?.setEditorColor("highlight", hex);
  }

  function setInkColor(hex: string) {
    inkColor.value = hex;
    if (tool.value === "freetext" || tool.value === "ink") session.value?.setEditorColor(tool.value, hex);
  }

  function setTextSize(size: number) {
    textSize.value = size;
    session.value?.setEditorSize("freetext", size);
  }

  function setInkWidth(width: number) {
    inkWidth.value = width;
    session.value?.setEditorSize("ink", width);
  }

  return {
    open,
    tool,
    highlightColor,
    inkColor,
    textSize,
    inkWidth,
    selectTool,
    toggle,
    reset,
    setHighlightColor,
    setInkColor,
    setTextSize,
    setInkWidth,
  };
}

export type PdfMarkupState = ReturnType<typeof usePdfMarkup>;
