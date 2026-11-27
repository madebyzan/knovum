<script setup lang="ts">
import {
  ArrowDown,
  ArrowUp,
  ChevronLeft,
  ChevronRight,
  Code2,
  Copy,
  Heading1,
  Heading2,
  Heading3,
  List,
  ListChecks,
  ListOrdered,
  MessageSquareQuote,
  Pilcrow,
  Quote,
  RefreshCw,
  Trash2,
} from "@lucide/vue";
import { shallowRef, type Component } from "vue";
import { useI18n } from "vue-i18n";

import { MENU_ITEM_BASE_CLASSES } from "../../../../shared/ui/ui-classes";
import UiKbd from "../../../../shared/ui/UiKbd.vue";
import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import { CalloutBlockIcon } from "../editor-icons";
import type { TurnIntoBlockKind } from "../tiptap-block-actions";

const emit = defineEmits<{
  askAi: [];
  askInChat: [];
  turnInto: [kind: TurnIntoBlockKind];
  moveUp: [];
  moveDown: [];
  duplicate: [];
  remove: [];
}>();

const { t } = useI18n();
const showTurnInto = shallowRef(false);

const TURN_INTO_ITEMS: readonly {
  readonly kind: TurnIntoBlockKind;
  readonly labelKey: string;
  readonly icon: Component;
}[] = [
  { kind: "paragraph", labelKey: "spaces.richtext.paragraph", icon: Pilcrow },
  { kind: "heading1", labelKey: "spaces.richtext.heading1", icon: Heading1 },
  { kind: "heading2", labelKey: "spaces.richtext.heading2", icon: Heading2 },
  { kind: "heading3", labelKey: "spaces.richtext.heading3", icon: Heading3 },
  { kind: "bulletList", labelKey: "spaces.richtext.bulletList", icon: List },
  { kind: "orderedList", labelKey: "spaces.richtext.orderedList", icon: ListOrdered },
  { kind: "taskList", labelKey: "spaces.richtext.taskList", icon: ListChecks },
  { kind: "blockquote", labelKey: "spaces.richtext.blockquote", icon: Quote },
  { kind: "callout", labelKey: "spaces.richtext.callout.title", icon: CalloutBlockIcon },
  { kind: "codeBlock", labelKey: "spaces.richtext.codeBlock.title", icon: Code2 },
];

const ITEM_CLASS = `${MENU_ITEM_BASE_CLASSES} w-full px-2 text-left text-fg-primary hover:bg-surface-control-hover focus-visible:bg-surface-control-hover`;
</script>

<template>
  <div class="w-52 p-1 select-none">
    <template v-if="showTurnInto">
      <button
        type="button"
        class="mb-1 flex h-interaction w-full items-center gap-1.5 rounded-md px-2 text-caption font-medium text-fg-secondary transition-colors hover:bg-surface-control-hover hover:text-fg-primary"
        @click="showTurnInto = false"
      >
        <ChevronLeft class="size-3.5" :stroke-width="1.6" />
        <span>{{ t("spaces.richtext.blockMenu.turnInto") }}</span>
      </button>
      <div class="max-h-64 overflow-y-auto">
        <button
          v-for="item in TURN_INTO_ITEMS"
          :key="item.kind"
          type="button"
          :class="ITEM_CLASS"
          @click="emit('turnInto', item.kind)"
        >
          <component :is="item.icon" class="size-3.5 text-fg-secondary" :stroke-width="1.6" />
          <span class="min-w-0 flex-1 truncate">{{ t(item.labelKey) }}</span>
        </button>
      </div>
    </template>
    <template v-else>
      <button type="button" :class="ITEM_CLASS" @click="emit('askAi')">
        <UiKnovumAiIcon class="size-3.5 text-fg-secondary" />
        <span class="min-w-0 flex-1">{{ t("spaces.richtext.blockMenu.askAi") }}</span>
        <UiKbd shortcut="$mod+J" />
      </button>
      <button type="button" :class="ITEM_CLASS" @click="showTurnInto = true">
        <RefreshCw class="size-3.5 text-fg-secondary" :stroke-width="1.6" />
        <span class="min-w-0 flex-1">{{ t("spaces.richtext.blockMenu.turnInto") }}</span>
        <ChevronRight class="size-3 text-fg-tertiary" :stroke-width="1.6" />
      </button>
      <button type="button" :class="ITEM_CLASS" @click="emit('askInChat')">
        <MessageSquareQuote class="size-3.5 text-fg-secondary" :stroke-width="1.6" />
        <span>{{ t("spaces.richtext.blockMenu.askInChat") }}</span>
      </button>
      <UiSeparator class="my-1" />
      <button type="button" :class="ITEM_CLASS" @click="emit('duplicate')">
        <Copy class="size-3.5 text-fg-secondary" :stroke-width="1.6" />
        <span>{{ t("spaces.richtext.blockMenu.duplicate") }}</span>
      </button>
      <button type="button" :class="ITEM_CLASS" @click="emit('moveUp')">
        <ArrowUp class="size-3.5 text-fg-secondary" :stroke-width="1.6" />
        <span class="min-w-0 flex-1">{{ t("spaces.richtext.blockMenu.moveUp") }}</span>
        <UiKbd shortcut="Alt+ArrowUp" />
      </button>
      <button type="button" :class="ITEM_CLASS" @click="emit('moveDown')">
        <ArrowDown class="size-3.5 text-fg-secondary" :stroke-width="1.6" />
        <span class="min-w-0 flex-1">{{ t("spaces.richtext.blockMenu.moveDown") }}</span>
        <UiKbd shortcut="Alt+ArrowDown" />
      </button>
      <UiSeparator class="my-1" />
      <button type="button" :class="[ITEM_CLASS, 'text-fg-danger']" @click="emit('remove')">
        <Trash2 class="size-3.5" :stroke-width="1.6" />
        <span>{{ t("spaces.richtext.blockMenu.delete") }}</span>
      </button>
    </template>
  </div>
</template>
