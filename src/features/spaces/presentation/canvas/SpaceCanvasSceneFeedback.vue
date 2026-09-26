<script setup lang="ts">
import { BookmarkCheck, LocateFixed, RefreshCw } from "@lucide/vue";
import { useTimeoutFn } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { SceneCue } from "../../session/session-state";
import { useSpaceStore } from "../../session/space-store";

const { t } = useI18n();
const store = useSpaceStore();
const { sceneCue } = storeToRefs(store);
const displayed = shallowRef<SceneCue>();
const captureFrame = shallowRef<SceneCue>();
const { start, stop } = useTimeoutFn(() => (displayed.value = undefined), 2200, { immediate: false });
const { start: startCapture, stop: stopCapture } = useTimeoutFn(() => (captureFrame.value = undefined), 460, {
  immediate: false,
});
const icon = computed(() => {
  if (displayed.value?.action === "opened") return LocateFixed;
  if (displayed.value?.action === "updated") return RefreshCw;
  return BookmarkCheck;
});
const label = computed(() => {
  const cue = displayed.value;
  if (!cue) return "";
  const key =
    cue.action === "opened"
      ? "spaces.scenes.opened"
      : cue.action === "updated"
        ? "spaces.scenes.updated"
        : "spaces.scenes.saved";
  return t(key, { title: cue.title });
});

watch(sceneCue, (cue) => {
  if (!cue) return;
  displayed.value = { ...cue };
  if (cue.action === "saved") {
    captureFrame.value = { ...cue };
    stopCapture();
    startCapture();
  }
  stop();
  start();
});
</script>

<template>
  <Transition
    enter-active-class="animate-in fade-in-0 zoom-in-95 duration-150 ease-standard motion-reduce:animate-none"
    leave-active-class="animate-out fade-out-0 zoom-out-95 slide-out-to-bottom-1 slide-out-to-left-1 duration-200 ease-standard motion-reduce:animate-none"
  >
    <div
      v-if="captureFrame"
      :key="captureFrame.nonce"
      class="pointer-events-none absolute inset-2 origin-bottom-left rounded-canvas-object border border-accent/35 bg-accent-subtle"
      aria-hidden="true"
    />
  </Transition>
  <Transition
    enter-active-class="animate-in fade-in-0 slide-in-from-top-1 duration-150 ease-standard motion-reduce:animate-none"
    leave-active-class="animate-out fade-out-0 duration-150 motion-reduce:animate-none"
  >
    <div
      v-if="displayed"
      class="pointer-events-none absolute top-4 left-1/2 flex h-8 -translate-x-1/2 items-center gap-2 rounded-control border border-border bg-overlay px-2.5 text-caption text-foreground"
      role="status"
    >
      <component :is="icon" class="size-(--knovum-icon-size-xs) shrink-0 text-accent" :stroke-width="1.6" />
      <span class="max-w-72 truncate font-medium">{{ label }}</span>
      <span class="text-micro text-foreground-tertiary tabular-nums">
        {{ Math.round(displayed.zoom * 100) }}%
      </span>
    </div>
  </Transition>
</template>
