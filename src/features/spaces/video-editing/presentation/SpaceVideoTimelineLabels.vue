<script setup lang="ts">
import { Captions, Film, Music2 } from "@lucide/vue";
defineProps<{ audioCount: number }>();
</script>
<template>
  <div class="border-r border-stroke-divider text-caption text-fg-secondary">
    <div class="h-8 border-b border-stroke-divider" />
    <div class="flex h-11 items-center gap-1 border-b border-stroke-divider px-2">
      <Captions class="size-3.5" />{{ $t("spaces.media.videoEditor.tracks.text") }}
    </div>
    <div class="flex h-24 items-center gap-1 border-b border-stroke-divider px-2">
      <Film class="size-3.5" />{{ $t("spaces.media.videoEditor.tracks.video") }}
    </div>
    <div
      v-for="(_, index) in Math.max(1, audioCount)"
      :key="index"
      class="flex h-8 items-center gap-1 border-b border-stroke-divider px-2"
    >
      <Music2 class="size-3.5" />{{ $t("spaces.media.videoEditor.tracks.audio") }} {{ index + 1 }}
    </div>
  </div>
</template>
