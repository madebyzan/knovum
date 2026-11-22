<script setup lang="ts">
import { Link2, Plus } from "@lucide/vue";
import { computed } from "vue";

import type {
  ConversationAttachmentCandidatePresentation,
  ConversationAttachmentPresentation,
} from "../conversation-attachment-presentation";
import SpaceConversationAttachment from "./SpaceConversationAttachment.vue";

const props = defineProps<{
  attachments: readonly ConversationAttachmentPresentation[];
  unresolvedFileNames: readonly string[];
  selectedCandidates?: readonly ConversationAttachmentCandidatePresentation[] | undefined;
  showSelectionHint?: boolean | undefined;
}>();
const emit = defineEmits<{
  discardReference: [name: string];
  remove: [key: string];
  preview: [key: string];
  addSelection: [];
}>();
const unresolvedAttachments = computed<ConversationAttachmentPresentation[]>(() =>
  props.unresolvedFileNames.map((name) => ({
    key: `unresolved:${name}`,
    kind: "file",
    title: name,
    isImage: false,
    previewSource: "",
  })),
);
const selectedCount = computed(() => props.selectedCandidates?.length ?? 0);
</script>

<template>
  <div v-if="showSelectionHint && selectedCount > 0" class="border-b border-stroke-divider/55 px-2 py-1.5">
    <button
      type="button"
      class="flex w-full min-w-0 items-center gap-1.5 rounded-md px-1.5 py-0.5 text-left text-caption-sm text-fg-secondary transition-colors duration-100 hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
      @click="emit('addSelection')"
    >
      <Link2 class="size-3 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
      <span class="min-w-0 flex-1 truncate">
        {{
          selectedCount === 1
            ? $t("spaces.conversation.attachments.addSelectedOne", {
                title: selectedCandidates![0]!.title,
              })
            : $t("spaces.conversation.attachments.addCurrentSelection", { count: selectedCount })
        }}
      </span>
      <Plus class="size-3 shrink-0 text-fg-tertiary" :stroke-width="1.8" />
    </button>
  </div>
  <div
    v-if="attachments.length || unresolvedAttachments.length"
    class="min-w-0 overflow-x-auto px-3 pt-3 pb-1 focus-visible:outline-2 focus-visible:outline-stroke-focus"
    role="group"
    tabindex="0"
    :aria-label="$t('spaces.conversation.attachments.attached')"
  >
    <div class="flex w-max gap-2">
      <SpaceConversationAttachment
        v-for="attachment in attachments"
        :key="attachment.key"
        :attachment="attachment"
        @remove="emit('remove', $event)"
        @preview="emit('preview', $event)"
      />
      <SpaceConversationAttachment
        v-for="attachment in unresolvedAttachments"
        :key="attachment.key"
        :attachment="attachment"
        unresolved
        @remove="emit('discardReference', attachment.title)"
      />
    </div>
  </div>
</template>
