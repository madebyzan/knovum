<script setup lang="ts">
import { BookmarkCheck, LocateFixed, RefreshCw } from "@lucide/vue";
import { useTimeoutFn } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { SceneCue } from "../../session/session-state";
import { useSpaceStore } from "../../session/space-store";

const { t } = useI18n();
const store = useSpaceStore();
const { sceneCue } = storeToRefs(store);
const displayed = ref<SceneCue>();
const { start: startTimer, stop: stopTimer } = useTimeoutFn(
  () => {
    displayed.value = undefined;
  },
  2200,
  { immediate: false },
);
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
      ? "spaces.navigation.sceneOpened"
      : cue.action === "updated"
        ? "spaces.navigation.sceneUpdated"
        : "spaces.navigation.sceneSaved";
  return t(key, { title: cue.title });
});

watch(sceneCue, (cue) => {
  if (!cue) return;
  displayed.value = { ...cue };
  stopTimer();
  startTimer();
});
</script>

<template>
  <Transition
    enter-active-class="animate-in fade-in-0 duration-150 motion-reduce:animate-none"
    leave-active-class="animate-out fade-out-0 duration-200 motion-reduce:animate-none"
  >
    <div
      v-if="displayed"
      class="pointer-events-none absolute inset-3 z-20 rounded-canvas-object ring-2 ring-accent/25 ring-inset"
    >
      <div
        class="absolute top-3 left-1/2 flex -translate-x-1/2 items-center gap-2 rounded-panel bg-overlay/95 px-3 py-2 text-caption text-foreground shadow-popover ring-1 ring-border backdrop-blur"
      >
        <component :is="icon" class="size-(--knovum-icon-size-sm) shrink-0 text-accent" :stroke-width="1.7" />
        <span class="max-w-72 truncate font-medium">{{ label }}</span>
        <span class="text-micro text-foreground-tertiary tabular-nums"
          >{{ Math.round(displayed.zoom * 100) }}%</span
        >
      </div>
    </div>
  </Transition>
</template>
