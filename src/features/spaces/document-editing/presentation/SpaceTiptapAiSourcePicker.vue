<script setup lang="ts">
import { AtSign, Check } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { EditorContextReference } from "../../../../shared/contracts/generated/EditorContextReference";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import { nodeIcon } from "../../canvas/presentation/node-icon";
import { filterSpaceAiReferences, type SpaceAiReferenceCandidate } from "../tiptap-space-references";

const props = defineProps<{
  candidates: readonly SpaceAiReferenceCandidate[];
  attached: readonly EditorContextReference[];
}>();
const emit = defineEmits<{ toggle: [candidate: SpaceAiReferenceCandidate] }>();
const { t } = useI18n();
const open = shallowRef(false);
const query = shallowRef("");

const attachedIds = computed(() => new Set(props.attached.map((item) => item.nodeId)));
const filtered = computed(() => filterSpaceAiReferences(props.candidates, query.value).slice(0, 12));
</script>

<template>
  <UiPopover
    v-model:open="open"
    align="start"
    side="bottom"
    :side-offset="6"
    class="w-72 overflow-hidden p-1.5"
  >
    <template #trigger>
      <UiIconButton
        data-testid="ai-panel-reference-picker"
        :label="t('spaces.richtext.ai.panel.addReference')"
        tooltip
        size="sm"
        :disabled="!candidates.length"
        :active="attached.length > 0"
      >
        <AtSign class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
    </template>

    <UiSearchInput
      v-model="query"
      :placeholder="t('spaces.richtext.ai.panel.searchReferences')"
      class="mb-1.5"
    />
    <div class="max-h-56 overflow-y-auto">
      <button
        v-for="item in filtered"
        :key="item.nodeId"
        type="button"
        class="flex h-interaction w-full items-center gap-2 rounded-md px-2 text-left text-label text-fg-primary transition-colors hover:bg-surface-control-hover"
        @click="emit('toggle', item)"
      >
        <component
          :is="nodeIcon(item.kind)"
          class="size-3.5 shrink-0 text-fg-secondary"
          :stroke-width="1.6"
        />
        <span class="min-w-0 flex-1 truncate">{{ item.title }}</span>
        <Check
          v-if="attachedIds.has(item.nodeId)"
          class="size-3.5 shrink-0 text-fg-accent"
          :stroke-width="1.75"
        />
      </button>
      <p v-if="!filtered.length" class="px-2 py-3 text-center text-caption text-fg-tertiary">
        {{ t("spaces.richtext.ai.panel.noReferences") }}
      </p>
    </div>
  </UiPopover>
</template>
