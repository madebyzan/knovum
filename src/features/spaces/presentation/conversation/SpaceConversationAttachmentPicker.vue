<script setup lang="ts">
import { ArrowLeft, ArrowRight, Check, FileUp, Link2, Plus } from "@lucide/vue";
import { search } from "fast-fuzzy";
import { computed, shallowRef, watch } from "vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";

interface Candidate {
  nodeId: string;
  title: string;
  previewSource: string;
  selected: boolean;
  attached: boolean;
}

const props = defineProps<{ candidates: readonly Candidate[]; pickingFiles: boolean }>();
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
  <UiPopover v-model:open="open" side="top" align="start" class="w-80 max-w-[calc(100vw-24px)] p-2">
    <template #trigger>
      <UiIconButton :label="$t('spaces.assistant.attachments.add')" size="sm">
        <Plus class="size-3.5" />
      </UiIconButton>
    </template>
    <template v-if="view === 'sources'">
      <p class="px-2 py-1.5 text-caption-sm font-medium text-fg-tertiary">
        {{ $t("spaces.assistant.attachments.source") }}
      </p>
      <button
        type="button"
        class="flex w-full items-center gap-3 rounded-md px-2 py-2.5 text-left hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
        @click="view = 'nodes'"
      >
        <span
          class="flex size-9 shrink-0 items-center justify-center rounded-md bg-surface-accent-subtle text-fg-accent"
        >
          <Link2 class="size-3.5" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="block text-caption font-medium">{{
            $t("spaces.assistant.attachments.citeCanvasNode")
          }}</span>
          <span class="block text-caption-sm text-fg-tertiary">{{
            $t("spaces.assistant.attachments.citeCanvasNodeHint")
          }}</span>
        </span>
        <ArrowRight class="size-3 shrink-0 text-fg-tertiary" />
      </button>
      <button
        type="button"
        class="flex w-full items-center gap-3 rounded-md px-2 py-2.5 text-left hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-50"
        :disabled="pickingFiles"
        @click="chooseFiles"
      >
        <span
          class="flex size-9 shrink-0 items-center justify-center rounded-md bg-surface-control text-fg-secondary"
        >
          <FileUp class="size-3.5" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="block text-caption font-medium">{{
            $t("spaces.assistant.attachments.uploadLocalFile")
          }}</span>
          <span class="block text-caption-sm text-fg-tertiary">{{
            $t("spaces.assistant.attachments.uploadLocalFileHint")
          }}</span>
        </span>
      </button>
    </template>
    <template v-else>
      <div class="flex items-center gap-1 px-1 pb-1">
        <UiIconButton :label="$t('common.back')" size="sm" @click="view = 'sources'">
          <ArrowLeft class="size-3.5" />
        </UiIconButton>
        <span class="text-caption font-medium">{{ $t("spaces.assistant.attachments.citeCanvasNode") }}</span>
      </div>
      <div class="px-1 pb-1">
        <UiSearchInput
          v-model="query"
          autofocus
          size="sm"
          :placeholder="$t('spaces.assistant.attachments.search')"
        />
      </div>
      <button
        type="button"
        class="flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-caption hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-50"
        :disabled="selectedCount === 0"
        @click="addSelected"
      >
        <Plus class="size-3 shrink-0" />
        {{ $t("spaces.assistant.attachments.addCurrentSelection", { count: selectedCount }) }}
      </button>
      <div class="mx-1 my-1 border-t border-stroke-divider" />
      <div class="max-h-60 overflow-y-auto">
        <button
          v-for="candidate in matching"
          :key="candidate.nodeId"
          type="button"
          class="flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-caption hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-50"
          :disabled="candidate.attached"
          @click="addOne(candidate.nodeId)"
        >
          <img
            v-if="candidate.previewSource"
            :src="candidate.previewSource"
            alt=""
            class="size-8 shrink-0 rounded-md object-cover"
          />
          <span
            v-else
            class="flex size-8 shrink-0 items-center justify-center rounded-md bg-surface-accent-subtle text-fg-accent"
          >
            <Link2 class="size-3" />
          </span>
          <span class="min-w-0 flex-1 truncate">{{ candidate.title }}</span>
          <Check v-if="candidate.attached" class="size-3 shrink-0" />
        </button>
        <p v-if="matching.length === 0" class="px-2 py-4 text-center text-caption text-fg-tertiary">
          {{ $t("spaces.assistant.attachments.none") }}
        </p>
      </div>
    </template>
  </UiPopover>
</template>
