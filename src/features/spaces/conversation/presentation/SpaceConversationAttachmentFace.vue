<script setup lang="ts">
import { File, FileImage, Link2 } from "@lucide/vue";
import { computed } from "vue";

import { attachmentFileType } from "../conversation-attachment-presentation";

// What an attachment looks like wherever the conversation shows one — waiting in the composer,
// sent with a message, or added by a reply. An image is its picture; a document is a file row
// that names it whole enough to recognise and says what kind of file it is, never a picture-sized
// grey tile with the name cut down to fit.
const props = defineProps<{
  title: string;
  image: boolean;
  thumbnail?: string | null;
  /** A canvas node rather than an uploaded file. */
  linked: boolean;
}>();
const fileType = computed(() => attachmentFileType(props.title));
</script>

<template>
  <span v-if="image" class="relative flex size-20 items-center justify-center bg-surface-muted">
    <img v-if="thumbnail" :src="thumbnail" alt="" class="size-full object-cover" />
    <FileImage v-else class="size-5 text-fg-tertiary" :stroke-width="1.5" />
    <span
      v-if="linked"
      class="absolute bottom-1 left-1 rounded-sm bg-surface-overlay/90 p-0.5 text-fg-accent"
    >
      <Link2 class="size-3" />
    </span>
  </span>
  <span v-else class="flex h-14 w-60 max-w-full min-w-0 items-center gap-2.5 px-2.5">
    <span
      class="flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-md bg-surface-muted"
    >
      <img v-if="thumbnail" :src="thumbnail" alt="" class="size-full object-cover" />
      <File v-else class="size-4.5 text-fg-secondary" :stroke-width="1.5" />
    </span>
    <span class="flex min-w-0 flex-1 flex-col gap-0.5 text-left">
      <span class="truncate text-caption font-medium text-fg-primary">{{ title }}</span>
      <span class="flex min-w-0 items-center gap-1 text-caption-sm text-fg-tertiary">
        <Link2 v-if="linked" class="size-3 shrink-0 text-fg-accent" />
        <span class="truncate">{{
          fileType ??
          $t(
            linked
              ? "spaces.conversation.attachments.canvasReference.shortLabel"
              : "spaces.conversation.attachments.localFile.shortLabel",
          )
        }}</span>
      </span>
    </span>
  </span>
</template>
