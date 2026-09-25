<script setup lang="ts">
import { Check, Plus } from "@lucide/vue";
import { computed, shallowRef } from "vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";

interface Candidate {
  nodeId: string;
  title: string;
  selected: boolean;
  attached: boolean;
}

const props = defineProps<{ candidates: readonly Candidate[] }>();
const emit = defineEmits<{ addAttachment: [nodeId: string]; addSelection: [] }>();
const open = shallowRef(false);
const query = shallowRef("");
const selectedCount = computed(
  () => props.candidates.filter(({ selected, attached }) => selected && !attached).length,
);
const matching = computed(() =>
  props.candidates.filter(({ title }) =>
    title.toLocaleLowerCase().includes(query.value.trim().toLocaleLowerCase()),
  ),
);

function addSelected() {
  emit("addSelection");
  open.value = false;
}

function addOne(nodeId: string) {
  emit("addAttachment", nodeId);
  open.value = false;
}
</script>

<template>
  <UiPopover v-model:open="open" side="top" align="start" class="w-72 max-w-[calc(100vw-24px)] p-1.5">
    <template #trigger>
      <UiIconButton :label="$t('spaces.assistant.addResource')" size="sm">
        <Plus class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
    </template>
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
      class="flex w-full items-center gap-2 rounded-control px-2 py-1.5 text-left text-caption text-foreground hover:bg-control-hover focus-visible:outline-2 focus-visible:outline-focus-ring disabled:text-foreground-tertiary"
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
        class="flex w-full items-center gap-2 rounded-control px-2 py-1.5 text-left text-caption hover:bg-control-hover focus-visible:outline-2 focus-visible:outline-focus-ring disabled:text-foreground-tertiary"
        :disabled="candidate.attached"
        @click="addOne(candidate.nodeId)"
      >
        <span class="min-w-0 flex-1 truncate">{{ candidate.title }}</span>
        <Check v-if="candidate.attached" class="size-(--knovum-icon-size-xs) shrink-0" />
      </button>
      <p v-if="matching.length === 0" class="px-2 py-4 text-center text-caption text-foreground-tertiary">
        {{ $t("spaces.assistant.noAttachableResources") }}
      </p>
    </div>
  </UiPopover>
</template>
