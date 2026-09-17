import { onBeforeUnmount, onMounted, type Ref } from "vue";

import type { CanvasPoint, CanvasViewport } from "../../contract/canvas-editor";
import { canvasPointFromScreen } from "../viewport/viewport-math";

interface ExternalContentDropOptions {
  host: Readonly<Ref<HTMLElement | null>>;
  viewport(): CanvasViewport;
  insertText(text: string, point: CanvasPoint): Promise<void>;
  insertUrl(url: string, point: CanvasPoint): Promise<void>;
  reportError(error: unknown): void;
}

const acceptsExternalText = (transfer: DataTransfer | null) =>
  Boolean(transfer?.types.includes("text/uri-list") || transfer?.types.includes("text/plain"));

export function useExternalContentDrop(options: ExternalContentDropOptions) {
  function dragOver(event: DragEvent) {
    if (acceptsExternalText(event.dataTransfer)) event.preventDefault();
  }

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
    const url = transfer
      .getData("text/uri-list")
      .split("\n")
      .find((line) => line && !line.startsWith("#"));
    const operation = url
      ? options.insertUrl(url, point)
      : options.insertText(transfer.getData("text/plain"), point);
    void operation.catch(options.reportError);
  }

  onMounted(() => {
    options.host.value?.addEventListener("dragover", dragOver);
    options.host.value?.addEventListener("drop", drop);
  });
  onBeforeUnmount(() => {
    options.host.value?.removeEventListener("dragover", dragOver);
    options.host.value?.removeEventListener("drop", drop);
  });
}
