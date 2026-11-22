import { convertFileSrc } from "@tauri-apps/api/core";
import { useResizeObserver } from "@vueuse/core";
import { computed, nextTick, onBeforeUnmount, shallowRef, watch, type Ref, type ShallowRef } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";

async function fetchDocxWithRenderer(path: string, signal: AbortSignal) {
  const [response, docx] = await Promise.all([
    fetch(convertFileSrc(path), { signal }),
    import("docx-preview"),
  ]);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return { buffer: await response.arrayBuffer(), docx };
}

/** Owns abortable `.docx` rendering via `docx-preview`, fit-to-width zoom, and DOM memory cleanup. */
export function useDocxPreview(options: {
  scroller: Readonly<ShallowRef<HTMLDivElement | null>>;
  host: Readonly<ShallowRef<HTMLDivElement | null>>;
  styleHost: Readonly<ShallowRef<HTMLDivElement | null>>;
  assetPath: () => string | undefined;
  active: Readonly<Ref<boolean>>;
  onRendered?: () => void;
}) {
  const rendering = shallowRef(false);
  const failure = shallowRef<string>();
  const zoom = shallowRef(1);
  const manualZoom = shallowRef(false);
  let renderToken = 0;
  let fetchController: AbortController | undefined;
  let renderedPath: string | undefined;

  const zoomPercent = computed(() => Math.round(zoom.value * 100));

  function clearHosts() {
    options.host.value?.replaceChildren();
    options.styleHost.value?.replaceChildren();
  }

  function fitToWidth() {
    const scroller = options.scroller.value;
    const page = options.host.value?.querySelector<HTMLElement>("section.knovum-docx");
    if (!scroller || !page) return;
    const pageWidth = page.offsetWidth;
    const available = scroller.clientWidth - 28;
    if (pageWidth <= 0 || available <= 0) return;
    zoom.value = Number(Math.min(1, Math.max(0.45, available / pageWidth)).toFixed(3));
  }

  function adjustZoom(delta: number) {
    manualZoom.value = true;
    zoom.value = Math.min(1.6, Math.max(0.45, Number((zoom.value + delta).toFixed(2))));
  }

  function resetFit() {
    manualZoom.value = false;
    fitToWidth();
  }

  async function renderIntoHosts(
    path: string,
    host: HTMLDivElement,
    styleHost: HTMLDivElement,
    token: number,
  ) {
    const controller = new AbortController();
    fetchController = controller;
    const { buffer, docx } = await fetchDocxWithRenderer(path, controller.signal);
    if (token !== renderToken || !host.isConnected) return;
    clearHosts();
    await docx.renderAsync(buffer, host, styleHost, {
      inWrapper: true,
      ignoreWidth: false,
      ignoreHeight: true,
      breakPages: true,
      className: "knovum-docx",
    });
    if (token !== renderToken) return;
    renderedPath = path;
    manualZoom.value = false;
    await nextTick();
    fitToWidth();
    options.onRendered?.();
  }

  async function render(path: string) {
    const host = options.host.value;
    const styleHost = options.styleHost.value;
    if (!host || !styleHost) return;
    fetchController?.abort();
    const token = ++renderToken;
    rendering.value = true;
    failure.value = undefined;
    try {
      await renderIntoHosts(path, host, styleHost, token);
    } catch (error) {
      if (!fetchController?.signal.aborted && token === renderToken) {
        renderedPath = undefined;
        failure.value = getErrorMessage(error);
      }
    } finally {
      if (token === renderToken) rendering.value = false;
    }
  }

  useResizeObserver(options.scroller, () => {
    if (!manualZoom.value && options.active.value) fitToWidth();
  });

  watch(
    [options.host, options.assetPath, options.active],
    ([host, path, isActive]) => {
      if (!isActive || !host || !path) return;
      if (path === renderedPath) {
        if (!manualZoom.value) void nextTick(fitToWidth);
        return;
      }
      void render(path);
    },
    { immediate: true, flush: "post" },
  );

  onBeforeUnmount(() => {
    fetchController?.abort();
    renderedPath = undefined;
    clearHosts();
  });

  return { rendering, failure, zoom, zoomPercent, adjustZoom, resetFit };
}
