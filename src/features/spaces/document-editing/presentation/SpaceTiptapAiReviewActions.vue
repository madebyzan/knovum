<script setup lang="ts">
import { ArrowDownToLine, Check, RotateCcw } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { CompareDiffIcon } from "../editor-icons";

defineProps<{
  isReview: boolean;
  hasSelection: boolean;
  showCompare: boolean;
}>();

const emit = defineEmits<{
  retry: [];
  toggleCompare: [];
  discard: [];
  insertBelow: [];
  accept: [];
}>();

const { t } = useI18n();
</script>

<template>
  <footer
    class="flex items-center justify-between gap-2 border-t border-stroke-divider bg-surface-muted/40 px-3 py-2"
  >
    <div class="flex items-center gap-1">
      <UiButton data-testid="tiptap-ai-retry" size="sm" variant="ghost" @click="emit('retry')">
        <RotateCcw class="size-3.5" :stroke-width="1.6" />{{ t("spaces.richtext.ai.panel.retry") }}
      </UiButton>
      <UiButton
        v-if="isReview && hasSelection"
        data-testid="tiptap-ai-compare-toggle"
        size="sm"
        :variant="showCompare ? 'secondary' : 'ghost'"
        @click="emit('toggleCompare')"
      >
        <CompareDiffIcon class="size-3.5" :stroke-width="1.6" />{{ t("spaces.richtext.ai.panel.compare") }}
      </UiButton>
      <UiButton data-testid="tiptap-ai-discard" size="sm" variant="ghost" @click="emit('discard')">
        {{ t("spaces.richtext.ai.panel.discard") }}
      </UiButton>
    </div>
    <div v-if="isReview" class="flex items-center gap-1.5">
      <UiButton
        v-if="hasSelection"
        data-testid="tiptap-ai-insert-below"
        size="sm"
        @click="emit('insertBelow')"
      >
        <ArrowDownToLine class="size-3.5" :stroke-width="1.6" />{{
          t("spaces.richtext.ai.panel.insertBelow")
        }}
      </UiButton>
      <UiButton data-testid="tiptap-ai-accept" size="sm" variant="primary" @click="emit('accept')">
        <Check class="size-3.5" :stroke-width="1.75" />{{
          hasSelection ? t("spaces.richtext.ai.panel.replace") : t("spaces.richtext.ai.panel.insert")
        }}
      </UiButton>
    </div>
  </footer>
</template>
