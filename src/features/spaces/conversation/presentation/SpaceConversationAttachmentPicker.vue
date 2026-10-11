<script setup lang="ts">
import { ArrowLeft, ArrowRight, Check, FileUp, Link2, Paperclip, Plus } from "@lucide/vue";
import { search } from "fast-fuzzy";
import { computed, shallowRef, watch } from "vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import type { ConversationAttachmentCandidatePresentation } from "../conversation-attachment-presentation";

const props = defineProps<{
  candidates: readonly ConversationAttachmentCandidatePresentation[];
  pickingFiles: boolean;
}>();
const emit = defineEmits<{
  addAttachment: [nodeId: string];
  addSelection: [];
  chooseFiles: [];
}>();
const open = shallowRef(false);
const view = shallowRef<"sources" | "nodes">("sources");
const query = shallowRef("");
const selectedCount = computed(
  () => props.candidates.filter(({ selected, attached }) => selected && !attached).length,
);
const matching = computed(() => {
  const needle = query.value.trim();
  return needle
    ? search(needle, [...props.candidates], { keySelector: ({ title }) => title })
    : props.candidates;
});

watch(open, (value) => {
  if (!value) {
    view.value = "sources";
    query.value = "";
  }
});

function addSelected() {
  emit("addSelection");
  open.value = false;
}

function addOne(nodeId: string) {
  emit("addAttachment", nodeId);
  open.value = false;
}

function chooseFiles() {
  open.value = false;
  emit("chooseFiles");
}
</script>

<template>
  <UiPopover v-model:open="open" side="top" align="start" class="w-64 max-w-[calc(100vw-24px)] p-1.5">
    <template #trigger>
      <UiIconButton :label="$t('spaces.conversation.attachments.add')" size="sm">
        <Paperclip class="size-3.5" />
      </UiIconButton>
    </template>
    <template v-if="view === 'sources'">
      <p class="px-2.5 pt-1 pb-1.5 text-caption-sm font-medium text-fg-tertiary">
        {{ $t("spaces.conversation.attachments.add") }}
      </p>
      <button
        type="button"
        class="flex min-h-12 w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left transition-colors duration-100 hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
        @click="view = 'nodes'"
      >
        <span
          class="flex size-7 shrink-0 items-center justify-center rounded-md bg-surface-control text-fg-secondary"
        >
          <Link2 class="size-4" :stroke-width="1.6" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="block truncate text-label font-medium text-fg-primary">{{
            $t("spaces.conversation.attachments.canvasReference.label")
          }}</span>
          <span class="block truncate text-caption-sm text-fg-tertiary">{{
            $t("spaces.conversation.attachments.canvasReference.hint")
          }}</span>
        </span>
        <ArrowRight class="size-3.5 shrink-0 text-fg-tertiary" />
      </button>
      <button
        type="button"
        class="flex min-h-12 w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left transition-colors duration-100 hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-40"
        :disabled="pickingFiles"
        @click="chooseFiles"
      >
        <span
          class="flex size-7 shrink-0 items-center justify-center rounded-md bg-surface-control text-fg-secondary"
        >
          <FileUp class="size-4" :stroke-width="1.6" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="block truncate text-label font-medium text-fg-primary">{{
            $t("spaces.conversation.attachments.localFile.label")
          }}</span>
          <span class="block truncate text-caption-sm text-fg-tertiary">{{
            $t("spaces.conversation.attachments.localFile.hint")
          }}</span>
        </span>
      </button>
    </template>
    <template v-else>
      <div class="flex items-center gap-1 px-1 pb-1">
        <UiIconButton :label="$t('common.back')" size="sm" @click="view = 'sources'">
          <ArrowLeft class="size-3.5" />
        </UiIconButton>
        <span class="text-caption font-medium">{{
          $t("spaces.conversation.attachments.canvasReference.label")
        }}</span>
      </div>
      <div class="px-1 pb-1">
        <UiSearchInput
          v-model="query"
          autofocus
          size="sm"
          :label="$t('spaces.conversation.attachments.search')"
          :placeholder="$t('spaces.conversation.attachments.search')"
        />
      </div>
      <button
        type="button"
        class="flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-caption hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-40"
        :disabled="selectedCount === 0"
        @click="addSelected"
      >
        <Plus class="size-3 shrink-0" />
        {{ $t("spaces.conversation.attachments.addCurrentSelection", { count: selectedCount }) }}
      </button>
      <div class="mx-1 my-1 border-t border-stroke-divider" />
      <div class="max-h-60 overflow-y-auto">
        <button
          v-for="candidate in matching"
          :key="candidate.nodeId"
          type="button"
          class="flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-caption hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-40"
          :disabled="candidate.attached"
          @click="addOne(candidate.nodeId)"
        >
          <img
            v-if="candidate.previewSource"
            :src="candidate.previewSource"
            alt=""
            class="size-6 shrink-0 rounded-sm object-cover"
          />
          <Link2 v-else class="size-4 shrink-0 text-fg-accent" :stroke-width="1.6" />
          <span class="min-w-0 flex-1 truncate">{{ candidate.title }}</span>
          <Check v-if="candidate.attached" class="size-3 shrink-0" />
        </button>
        <p v-if="matching.length === 0" class="px-2 py-4 text-center text-caption text-fg-tertiary">
          {{ $t("spaces.conversation.attachments.none") }}
        </p>
      </div>
    </template>
  </UiPopover>
</template>
