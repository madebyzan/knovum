<script setup lang="ts">
import { AlertCircle, AlertTriangle, Check, Info, Lightbulb, OctagonAlert } from "@lucide/vue";
import { NodeViewContent, NodeViewWrapper, type NodeViewProps } from "@tiptap/vue-3";
import { computed, type Component } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import { CALLOUT_KINDS, normalizeCalloutKind, type CalloutKind } from "../tiptap-callout";

const props = defineProps<NodeViewProps>();
const { t } = useI18n();

const ICON_BY_KIND: Record<CalloutKind, Component> = {
  note: Info,
  tip: Lightbulb,
  important: AlertCircle,
  warning: AlertTriangle,
  caution: OctagonAlert,
};

const LABEL_KEY_BY_KIND: Record<CalloutKind, string> = {
  note: "spaces.richtext.callout.kinds.note",
  tip: "spaces.richtext.callout.kinds.tip",
  important: "spaces.richtext.callout.kinds.important",
  warning: "spaces.richtext.callout.kinds.warning",
  caution: "spaces.richtext.callout.kinds.caution",
};

const kind = computed<CalloutKind>(() => normalizeCalloutKind(props.node.attrs.kind));

function selectKind(next: CalloutKind) {
  props.updateAttributes({ kind: next });
}
</script>

<template>
  <NodeViewWrapper
    data-type="callout"
    :data-callout-kind="kind"
    class="space-callout my-2 flex items-start gap-3 rounded-panel border px-4 py-3"
  >
    <div contenteditable="false" class="mt-[1px] shrink-0 select-none">
      <UiDropdownMenu align="start" :side-offset="6" class="min-w-36">
        <template #trigger>
          <button
            type="button"
            data-testid="callout-kind-trigger"
            :disabled="!editor.isEditable"
            :aria-label="t(LABEL_KEY_BY_KIND[kind])"
            :title="t(LABEL_KEY_BY_KIND[kind])"
            class="space-callout-icon grid size-6 place-items-center rounded-md transition-colors duration-100 ease-standard outline-none hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus disabled:pointer-events-none"
          >
            <component :is="ICON_BY_KIND[kind]" class="size-4" :stroke-width="1.75" />
          </button>
        </template>
        <UiDropdownMenuItem v-for="option in CALLOUT_KINDS" :key="option" @select="selectKind(option)">
          <component :is="ICON_BY_KIND[option]" class="size-3.5 shrink-0" :stroke-width="1.6" />
          <span class="min-w-0 flex-1">{{ t(LABEL_KEY_BY_KIND[option]) }}</span>
          <Check v-if="kind === option" class="ml-auto size-3.5 text-fg-accent" :stroke-width="1.75" />
        </UiDropdownMenuItem>
      </UiDropdownMenu>
    </div>
    <NodeViewContent class="space-callout-body min-w-0 flex-1" />
  </NodeViewWrapper>
</template>
