<script setup lang="ts">
import { File, FileImage, FileText, Link2, X } from "@lucide/vue";
import { ref } from "vue";

defineProps<{
  attachment: { key: string; kind: "node" | "file"; title: string; isImage: boolean; previewSource: string };
}>();
const emit = defineEmits<{ remove: [key: string] }>();
const imageWidth = ref(96);

function sizeImage(event: Event) {
  const image = event.target as HTMLImageElement;
  imageWidth.value = Math.max(64, Math.min(144, Math.round((96 * image.naturalWidth) / image.naturalHeight)));
}
</script>

<template>
  <div
    class="group relative flex h-24 min-w-16 flex-col overflow-hidden rounded-lg border border-stroke-divider bg-surface-raised transition-[border-color,box-shadow] duration-150 hover:border-stroke-default-hover hover:shadow-control focus-within:border-stroke-focus"
    :class="{ 'w-36': !attachment.isImage }"
    :style="attachment.isImage ? { width: `${imageWidth}px` } : undefined"
    :title="attachment.title"
    role="group"
    :aria-label="`${$t(attachment.kind === 'node' ? 'spaces.assistant.attachments.canvasNodeShort' : 'spaces.assistant.attachments.localFileShort')}: ${attachment.title}`"
  >
    <div class="flex min-h-0 flex-1 items-center justify-center overflow-hidden bg-surface-muted">
      <img
        v-if="attachment.previewSource"
        :src="attachment.previewSource"
        alt=""
        class="size-full object-contain"
        @load="attachment.isImage && sizeImage($event)"
      />
      <FileImage v-else-if="attachment.isImage" class="size-5 text-fg-tertiary" :stroke-width="1.5" />
      <File v-else class="size-5 text-fg-tertiary" :stroke-width="1.5" />
    </div>
    <div v-if="!attachment.isImage" class="flex h-7 min-w-0 shrink-0 items-center gap-1.5 px-2">
      <Link2 v-if="attachment.kind === 'node'" class="size-3.5 shrink-0 text-fg-accent" />
      <FileText v-else class="size-3.5 shrink-0 text-fg-secondary" />
      <span class="min-w-0 truncate text-caption font-medium text-fg-primary">{{ attachment.title }}</span>
    </div>
    <button
      type="button"
      class="attachment-remove absolute top-1 right-1 flex size-6 items-center justify-center rounded-md bg-surface-overlay/95 text-fg-secondary opacity-0 transition-opacity duration-150 group-hover:opacity-100 group-focus-within:opacity-100 hover:bg-surface-control-hover hover:text-fg-primary focus-visible:opacity-100 focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :aria-label="$t('spaces.assistant.attachments.remove', { name: attachment.title })"
      @click="emit('remove', attachment.key)"
    >
      <X class="size-3.5" />
    </button>
  </div>
</template>

<style scoped>
@media (hover: none) {
  .attachment-remove { opacity: 1; }
}
</style>
