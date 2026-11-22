<script setup lang="ts">
import { onBeforeUnmount, watch } from "vue";

import type { PdfReaderSession } from "./pdf-editor-params";

const MAX_CONCURRENT_RENDERS = 2;

const props = defineProps<{
  session?: PdfReaderSession | undefined;
  pageCount: number;
  currentPage: number;
  highlightedPages: ReadonlySet<number>;
}>();

const emit = defineEmits<{ select: [page: number] }>();

const canvases = new Map<number, HTMLCanvasElement>();
const visiblePages = new Set<number>();
const rendered = new Set<number>();
const inFlight = new Set<number>();
let observer: IntersectionObserver | undefined;
let generation = 0;

function drainRenderQueue() {
  const session = props.session;
  if (!session) return;
  for (const page of visiblePages) {
    if (inFlight.size >= MAX_CONCURRENT_RENDERS) return;
    if (rendered.has(page) || inFlight.has(page)) continue;
    const canvas = canvases.get(page);
    if (!canvas) continue;
    const token = generation;
    inFlight.add(page);
    void session
      .renderThumbnail(page, canvas, 152)
      .then(() => {
        if (token === generation) rendered.add(page);
        return undefined;
      })
      .catch(() => undefined)
      .finally(() => {
        inFlight.delete(page);
        if (token === generation) drainRenderQueue();
      });
  }
}

function ensureObserver() {
  if (observer || typeof IntersectionObserver === "undefined") return;
  observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        const page = Number((entry.target as HTMLElement).dataset.thumbPage);
        if (!Number.isInteger(page)) continue;
        if (entry.isIntersecting) visiblePages.add(page);
        else visiblePages.delete(page);
      }
      drainRenderQueue();
    },
    { rootMargin: "180px" },
  );
}

function registerCanvas(page: number, el: unknown) {
  const prev = canvases.get(page);
  if (prev && observer) observer.unobserve(prev);
  if (el instanceof HTMLCanvasElement) {
    canvases.set(page, el);
    ensureObserver();
    observer?.observe(el);
  } else {
    canvases.delete(page);
    visiblePages.delete(page);
  }
}

watch(
  () => props.session,
  () => {
    generation += 1;
    rendered.clear();
    inFlight.clear();
    if (!props.session) return;
    ensureObserver();
    for (const canvas of canvases.values()) {
      observer?.unobserve(canvas);
      observer?.observe(canvas);
    }
  },
);

onBeforeUnmount(() => {
  generation += 1;
  observer?.disconnect();
  observer = undefined;
  for (const canvas of canvases.values()) {
    canvas.width = 0;
    canvas.height = 0;
  }
  canvases.clear();
  visiblePages.clear();
  rendered.clear();
  inFlight.clear();
});
</script>

<template>
  <div class="space-y-3 p-2.5">
    <button
      v-for="page in pageCount"
      :key="page"
      type="button"
      class="group flex w-full flex-col items-center gap-1 text-center"
      @click="emit('select', page)"
    >
      <div
        class="relative w-full overflow-hidden rounded-[4px] border bg-white shadow-2xs transition-all"
        :class="
          page === currentPage
            ? 'border-surface-accent ring-2 ring-surface-accent/35'
            : 'border-stroke-divider group-hover:border-stroke-strong'
        "
      >
        <canvas
          :ref="(el) => registerCanvas(page, el)"
          :data-thumb-page="page"
          class="block aspect-[3/4] w-full object-contain"
        />
        <span
          v-if="highlightedPages.has(page)"
          class="absolute top-1.5 right-1.5 size-2 rounded-full bg-amber-400 ring-2 ring-white"
        />
      </div>
      <span
        class="text-[11px] tabular-nums"
        :class="page === currentPage ? 'font-semibold text-fg-primary' : 'text-fg-tertiary'"
      >
        {{ page }}
      </span>
    </button>
  </div>
</template>
