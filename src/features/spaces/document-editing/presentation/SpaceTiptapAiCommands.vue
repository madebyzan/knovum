<script setup lang="ts">
import { ChevronLeft, ChevronRight, Languages, MessageSquareMore } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { TranslationTarget } from "../../../../foundation/ai/translation";
import type { EditorTransformAction } from "../../../../shared/contracts/generated/EditorTransformAction";
import { AI_INLINE_COMMANDS } from "../tiptap-writing-catalog";
import type { SpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";

const props = defineProps<{
  writing: SpaceTiptapWritingTools;
  hasSelection: boolean;
  selectedIndex: number;
}>();

const emit = defineEmits<{
  highlight: [index: number];
  runAction: [action: EditorTransformAction];
  runTranslation: [target: TranslationTarget];
}>();

const { t } = useI18n();
const subView = shallowRef<"root" | "tone" | "translate">("root");

const commands = computed(() =>
  AI_INLINE_COMMANDS.filter((item) => !item.needsSelection || props.hasSelection),
);

const ITEM_CLASS =
  "flex h-interaction w-full items-center gap-2.5 rounded-lg px-2.5 text-left text-label text-fg-primary transition-colors outline-none hover:bg-surface-control-hover data-[highlighted=true]:bg-surface-control-hover";

function executeHighlighted(): boolean {
  if (subView.value !== "root") return false;
  const item = commands.value[props.selectedIndex];
  if (!item) return false;
  emit("runAction", item.action);
  return true;
}

defineExpose({
  count: computed(() => (subView.value === "root" ? commands.value.length : 0)),
  executeHighlighted,
  resetSubView: () => {
    subView.value = "root";
  },
});
</script>

<template>
  <div class="border-t border-stroke-divider p-1.5 select-none">
    <template v-if="subView === 'tone'">
      <button
        type="button"
        class="mb-1 flex h-7 w-full items-center gap-1 rounded-md px-2 text-caption font-medium text-fg-secondary transition-colors hover:bg-surface-control-hover hover:text-fg-primary"
        @click="subView = 'root'"
      >
        <ChevronLeft class="size-3.5" />{{ t("spaces.richtext.ai.actions.adjustTone") }}
      </button>
      <div class="grid max-h-48 grid-cols-2 gap-0.5 overflow-y-auto">
        <button
          v-for="option in writing.tones.value"
          :key="option[0]"
          type="button"
          :class="ITEM_CLASS"
          @click="emit('runAction', option[0])"
        >
          <span class="truncate">{{ option[1] }}</span>
        </button>
      </div>
    </template>

    <template v-else-if="subView === 'translate'">
      <button
        type="button"
        class="mb-1 flex h-7 w-full items-center gap-1 rounded-md px-2 text-caption font-medium text-fg-secondary transition-colors hover:bg-surface-control-hover hover:text-fg-primary"
        @click="subView = 'root'"
      >
        <ChevronLeft class="size-3.5" />{{ t("spaces.richtext.ai.actions.translate") }}
      </button>
      <div class="grid max-h-48 grid-cols-2 gap-0.5 overflow-y-auto">
        <button
          v-for="option in writing.translations.value"
          :key="option[0]"
          type="button"
          :class="ITEM_CLASS"
          @click="emit('runTranslation', option[0])"
        >
          <span class="truncate">{{ option[1] }}</span>
        </button>
      </div>
    </template>

    <template v-else>
      <div class="max-h-56 overflow-y-auto">
        <button
          v-for="(item, index) in commands"
          :key="item.action"
          type="button"
          :data-highlighted="index === selectedIndex ? 'true' : undefined"
          :class="ITEM_CLASS"
          @mouseenter="emit('highlight', index)"
          @click="emit('runAction', item.action)"
        >
          <component :is="item.icon" class="size-3.5 shrink-0 text-fg-secondary" :stroke-width="1.6" />
          <span class="min-w-0 flex-1 truncate">{{ t(item.labelKey) }}</span>
        </button>
        <template v-if="hasSelection">
          <div class="my-1 h-px bg-stroke-divider" />
          <button type="button" :class="ITEM_CLASS" @click="subView = 'tone'">
            <MessageSquareMore class="size-3.5 shrink-0 text-fg-secondary" :stroke-width="1.7" />
            <span class="min-w-0 flex-1">{{ t("spaces.richtext.ai.actions.adjustTone") }}</span>
            <ChevronRight class="size-3 text-fg-tertiary" />
          </button>
          <button type="button" :class="ITEM_CLASS" @click="subView = 'translate'">
            <Languages class="size-3.5 shrink-0 text-fg-secondary" :stroke-width="1.7" />
            <span class="min-w-0 flex-1">{{ t("spaces.richtext.ai.actions.translate") }}</span>
            <ChevronRight class="size-3 text-fg-tertiary" />
          </button>
        </template>
      </div>
    </template>
  </div>
</template>
