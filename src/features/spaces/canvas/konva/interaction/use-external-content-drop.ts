import { useEventListener } from "@vueuse/core";
import type { Ref } from "vue";

import type { CanvasPoint, CanvasViewport } from "../../contracts/canvas-editor-port";
import { readPassageTransfer, type CanvasPassage } from "../../contracts/passage-transfer";
import { canvasPointFromScreen } from "../viewport/viewport-math";

interface ExternalContentDropOptions {
  host: Readonly<Ref<HTMLElement | null>>;
  viewport(): CanvasViewport;
  insertText(text: string, point: CanvasPoint): Promise<void>;
  insertPassage(passage: CanvasPassage, point: CanvasPoint): Promise<void>;
  insertUrl(url: string, point: CanvasPoint): Promise<void>;
  reportError(error: unknown): void;
}

function acceptsExternalText(transfer: DataTransfer | null) {
  return Boolean(transfer?.types.includes("text/uri-list") || transfer?.types.includes("text/plain"));
}

function dragOver(event: DragEvent) {
  if (acceptsExternalText(event.dataTransfer)) event.preventDefault();
}

export function useExternalContentDrop(options: ExternalContentDropOptions) {
  function drop(event: DragEvent) {
    const transfer = event.dataTransfer;
    if (!acceptsExternalText(transfer)) return;
    event.preventDefault();
    const host = options.host.value;
    if (!host || !transfer) return;
    const bounds = host.getBoundingClientRect();
    const point = canvasPointFromScreen(
      { x: event.clientX - bounds.left, y: event.clientY - bounds.top },
      options.viewport(),
    );
    const passage = readPassageTransfer(transfer);
    if (passage) {
      void options.insertPassage(passage, point).catch(options.reportError);
      return;
    }
    const url = transfer
      .getData("text/uri-list")
      .split("\n")
      .find((line) => line && !line.startsWith("#"));
    const operation = url
      ? options.insertUrl(url, point)
      : options.insertText(transfer.getData("text/plain"), point);
    void operation.catch(options.reportError);
  }

  const host = () => options.host.value;
  useEventListener(host, "dragover", dragOver);
  useEventListener(host, "drop", drop);
}
