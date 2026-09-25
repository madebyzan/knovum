<script setup lang="ts">
import { ArrowLeft, Check, FileUp, Link2, Plus } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";

interface Candidate {
  nodeId: string;
  title: string;
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
const matching = computed(() =>
  props.candidates.filter(({ title }) =>
    title.toLocaleLowerCase().includes(query.value.trim().toLocaleLowerCase()),
  ),
);

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
  <UiPopover v-model:open="open" side="top" align="start" class="w-72 max-w-[calc(100vw-24px)] p-1.5">
    <template #trigger>
      <UiIconButton :label="$t('spaces.assistant.addResource')" size="sm">
        <Plus class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
    </template>
    <template v-if="view === 'sources'">
      <p class="px-2 py-1 text-micro font-medium text-foreground-tertiary">
        {{ $t("spaces.assistant.attachmentSource") }}
      </p>
      <button
        type="button"
        class="flex w-full items-center gap-2 rounded-control px-2 py-2 text-left text-caption hover:bg-control-hover focus-visible:outline-2 focus-visible:outline-focus-ring"
        @click="view = 'nodes'"
      >
        <Link2 class="size-(--knovum-icon-size-sm) shrink-0 text-accent" />
        <span class="min-w-0">
          <span class="block text-caption font-medium">{{ $t("spaces.assistant.citeCanvasNode") }}</span>
          <span class="block text-micro text-foreground-tertiary">{{
            $t("spaces.assistant.citeCanvasNodeHint")
          }}</span>
        </span>
      </button>
      <button
        type="button"
        class="flex w-full items-center gap-2 rounded-control px-2 py-2 text-left text-caption hover:bg-control-hover focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-50"
        :disabled="pickingFiles"
        @click="chooseFiles"
      >
        <FileUp class="size-(--knovum-icon-size-sm) shrink-0 text-accent" />
        <span class="min-w-0">
          <span class="block text-caption font-medium">{{ $t("spaces.assistant.uploadLocalFile") }}</span>
          <span class="block text-micro text-foreground-tertiary">{{
            $t("spaces.assistant.uploadLocalFileHint")
          }}</span>
        </span>
      </button>
    </template>
    <template v-else>
      <div class="flex items-center gap-1 px-1 pb-1">
        <UiIconButton :label="$t('common.back')" size="sm" @click="view = 'sources'">
          <ArrowLeft class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
        <span class="text-caption font-medium">{{ $t("spaces.assistant.citeCanvasNode") }}</span>
      </div>
      <div class="px-1 pb-1">
        <UiSearchInput
          v-model="query"
          autofocus
          size="sm"
          :placeholder="$t('spaces.assistant.searchResources')"
        />
      </div>
      <button
        type="button"
        class="flex w-full items-center gap-2 rounded-control px-2 py-2 text-left text-caption hover:bg-control-hover focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-50"
        :disabled="selectedCount === 0"
        @click="addSelected"
      >
        <Plus class="size-(--knovum-icon-size-xs) shrink-0" />
        {{ $t("spaces.assistant.addCurrentSelection", { count: selectedCount }) }}
      </button>
      <div class="mx-1 my-1 border-t border-divider" />
      <div class="max-h-60 overflow-y-auto">
        <button
          v-for="candidate in matching"
          :key="candidate.nodeId"
          type="button"
          class="flex w-full items-center gap-2 rounded-control px-2 py-2 text-left text-caption hover:bg-control-hover focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-50"
          :disabled="candidate.attached"
          @click="addOne(candidate.nodeId)"
        >
          <Link2 class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary" />
          <span class="min-w-0 flex-1 truncate">{{ candidate.title }}</span>
          <Check v-if="candidate.attached" class="size-(--knovum-icon-size-xs) shrink-0" />
        </button>
        <p v-if="matching.length === 0" class="px-2 py-4 text-center text-caption text-foreground-tertiary">
          {{ $t("spaces.assistant.noAttachableResources") }}
        </p>
      </div>
    </template>
  </UiPopover>
</template>
