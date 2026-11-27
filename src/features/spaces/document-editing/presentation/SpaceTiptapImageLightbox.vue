<script setup lang="ts">
import { ChevronLeft, ChevronRight, X, ZoomIn, ZoomOut } from "@lucide/vue";
import { computed, onBeforeUnmount, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { LightboxImageItem } from "../../../../shared/ui/image-group-layout";

const props = defineProps<{
  open: boolean;
  src?: string;
  caption?: string;
  items?: readonly LightboxImageItem[];
  initialIndex?: number;
}>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();

const MIN_ZOOM = 0.5;
const MAX_ZOOM = 2.5;
const zoom = shallowRef(1);
const activeIndex = shallowRef(0);

const gallery = computed<readonly LightboxImageItem[]>(() =>
  props.items?.length ? props.items : [{ src: props.src ?? "", caption: props.caption ?? "" }],
);
const currentItem = computed<LightboxImageItem>(
  () => gallery.value[activeIndex.value] ?? gallery.value[0] ?? { src: "", caption: "" },
);
const hasMultiple = computed(() => gallery.value.length > 1);

function adjustZoom(delta: number) {
  zoom.value = Math.round(Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom.value + delta)) * 100) / 100;
}

function stepSlide(delta: number) {
  const len = gallery.value.length;
  if (len <= 1) return;
  activeIndex.value = (activeIndex.value + delta + len) % len;
  zoom.value = 1;
}

function selectSlide(index: number) {
  activeIndex.value = Math.max(0, Math.min(index, gallery.value.length - 1));
  zoom.value = 1;
}

function handleWheel(event: WheelEvent) {
  if (!event.deltaY) return;
  adjustZoom(event.deltaY < 0 ? 0.15 : -0.15);
}

function handleKeydown(event: KeyboardEvent) {
  const actions: Record<string, () => void> = {
    Escape: () => emit("close"),
    ArrowLeft: () => stepSlide(-1),
    ArrowRight: () => stepSlide(1),
    "+": () => adjustZoom(0.25),
    "=": () => adjustZoom(0.25),
    "-": () => adjustZoom(-0.25),
    "0": () => (zoom.value = 1),
  };
  const run = actions[event.key];
  if (!run) return;
  event.preventDefault();
  event.stopPropagation();
  run();
}

watch(
  () => [props.open, props.initialIndex] as const,
  ([visible, start]) => {
    zoom.value = 1;
    activeIndex.value = Math.max(0, Math.min(start ?? 0, Math.max(0, gallery.value.length - 1)));
    if (visible) globalThis.addEventListener("keydown", handleKeydown, true);
    else globalThis.removeEventListener("keydown", handleKeydown, true);
  },
);

onBeforeUnmount(() => globalThis.removeEventListener("keydown", handleKeydown, true));
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      data-testid="image-lightbox"
      role="dialog"
      aria-modal="true"
      class="fixed inset-0 z-50 flex flex-col items-center justify-center bg-neutral-950/90 p-8 backdrop-blur-md select-none"
      @click.self="emit('close')"
      @wheel.prevent="handleWheel"
    >
      <div
        v-if="hasMultiple"
        class="fixed top-4 left-5 rounded-lg bg-neutral-900/80 px-2.5 py-1 text-caption-sm font-medium text-white/85 tabular-nums ring-1 ring-white/12 backdrop-blur-md"
      >
        {{ activeIndex + 1 }} / {{ gallery.length }}
      </div>

      <div
        class="fixed top-4 right-5 flex items-center gap-0.5 rounded-lg bg-neutral-900/80 p-1 text-white/85 shadow-lg ring-1 ring-white/12 backdrop-blur-md"
      >
        <button
          type="button"
          :aria-label="t('spaces.richtext.image.zoomOut')"
          class="grid size-7 place-items-center rounded-md transition-colors hover:bg-white/12 hover:text-white"
          @click="adjustZoom(-0.25)"
        >
          <ZoomOut class="size-3.5" :stroke-width="1.65" />
        </button>
        <button
          type="button"
          class="h-7 min-w-12 rounded-md px-1.5 text-caption-sm font-medium text-white/85 tabular-nums transition-colors hover:bg-white/12 hover:text-white"
          @click="zoom = 1"
        >
          {{ Math.round(zoom * 100) }}%
        </button>
        <button
          type="button"
          :aria-label="t('spaces.richtext.image.zoomIn')"
          class="grid size-7 place-items-center rounded-md transition-colors hover:bg-white/12 hover:text-white"
          @click="adjustZoom(0.25)"
        >
          <ZoomIn class="size-3.5" :stroke-width="1.65" />
        </button>
        <span class="mx-0.5 h-3.5 w-px bg-white/15" />
        <button
          type="button"
          data-testid="image-lightbox-close"
          :aria-label="t('common.close')"
          class="grid size-7 place-items-center rounded-md transition-colors hover:bg-white/12 hover:text-white"
          @click="emit('close')"
        >
          <X class="size-3.5" :stroke-width="1.75" />
        </button>
      </div>

      <button
        v-if="hasMultiple"
        type="button"
        data-testid="image-lightbox-prev"
        :aria-label="t('spaces.richtext.image.previous')"
        class="fixed top-1/2 left-5 grid size-10 -translate-y-1/2 place-items-center rounded-full bg-neutral-900/75 text-white/85 ring-1 ring-white/14 backdrop-blur-md transition-colors hover:bg-neutral-800/90 hover:text-white"
        @click.stop="stepSlide(-1)"
      >
        <ChevronLeft class="size-5" :stroke-width="1.75" />
      </button>
      <button
        v-if="hasMultiple"
        type="button"
        data-testid="image-lightbox-next"
        :aria-label="t('spaces.richtext.image.next')"
        class="fixed top-1/2 right-5 grid size-10 -translate-y-1/2 place-items-center rounded-full bg-neutral-900/75 text-white/85 ring-1 ring-white/14 backdrop-blur-md transition-colors hover:bg-neutral-800/90 hover:text-white"
        @click.stop="stepSlide(1)"
      >
        <ChevronRight class="size-5" :stroke-width="1.75" />
      </button>

      <img
        :src="currentItem.src"
        :alt="currentItem.caption"
        draggable="false"
        :style="{ transform: `scale(${zoom})` }"
        :class="zoom > 1.05 ? 'cursor-zoom-out' : 'cursor-zoom-in'"
        class="max-h-[76vh] max-w-[84vw] rounded-lg object-contain shadow-2xl transition-transform duration-200 ease-standard"
        @click.stop="zoom = zoom > 1.05 ? 1 : 1.75"
      />
      <p
        v-if="currentItem.caption.trim()"
        class="mt-3.5 max-w-2xl text-center text-body leading-relaxed whitespace-pre-wrap text-white/82 select-text"
      >
        {{ currentItem.caption }}
      </p>

      <div
        v-if="hasMultiple"
        class="fixed bottom-5 flex max-w-[80vw] items-center gap-1.5 overflow-x-auto rounded-lg bg-neutral-900/78 p-1.5 ring-1 ring-white/12 backdrop-blur-md"
      >
        <button
          v-for="(item, index) in gallery"
          :key="`${item.src}-${index}`"
          type="button"
          class="size-11 shrink-0 overflow-hidden rounded-md transition-all"
          :class="index === activeIndex ? 'opacity-100 ring-2 ring-white' : 'opacity-55 hover:opacity-90'"
          @click.stop="selectSlide(index)"
        >
          <img :src="item.src" :alt="item.caption" class="size-full object-cover" draggable="false" />
        </button>
      </div>
    </div>
  </Teleport>
</template>
