<script setup lang="ts">
import { Focus, Info, Scissors } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed, toRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { CANVAS_ACTION_BAR_CLASSES, CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
import { canvasBarPlacement } from "./canvas-bar-density-policy";
import { nodeOpenIcons, nodeOpenLabels } from "./canvas-node-open-actions";
import { nodeActionLabelClass, nodeActionLabelState } from "./node-action-label";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceCanvasTextFormatBar from "./SpaceCanvasTextFormatBar.vue";
import SpaceNodeBarCommonActions from "./SpaceNodeBarCommonActions.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";
import { useNodeActionBarDensity } from "./use-node-action-bar-density";

const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  node?: SpaceCanvasNode | undefined;
  availableWidth: number;
}>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t, locale } = useI18n();
const bar = useTemplateRef<HTMLElement>("bar");
const densityIdentity = computed(
  () =>
    `${locale.value}:${props.node?.data.kind}:${props.actions.state.value.canOpen}:${Boolean(props.actions.selectedVideoNode.value)}:${Boolean(props.actions.state.value.section)}`,
);
const { compact, minimal, ready } = useNodeActionBarDensity(
  bar,
  toRef(props, "availableWidth"),
  densityIdentity,
);
const primaryLabel = computed(() => {
  if (props.actions.state.value.section && props.actions.state.value.content.length)
    return t("spaces.navigation.focusSelection");
  const kind = props.node?.data.kind;
  return t(kind && props.actions.state.value.canOpen ? nodeOpenLabels[kind] : "spaces.navigation.inspect");
});
const primaryIcon = computed(() => {
  if (props.actions.state.value.section && props.actions.state.value.content.length) return Focus;
  const kind = props.node?.data.kind;
  return kind && props.actions.state.value.canOpen ? nodeOpenIcons[kind] : Info;
});
const showPrimary = computed(() => {
  if (props.actions.state.value.section && props.actions.state.value.content.length) return true;
  const kind = props.node?.data.kind;
  return Boolean(
    kind && props.actions.state.value.canOpen && nodeOpenLabels[kind] !== "spaces.navigation.inspect",
  );
});
const textKind = computed(() => {
  const kind = props.node?.data.kind;
  return kind === "note" || kind === "text" ? kind : undefined;
});

function runPrimaryAction() {
  if (props.actions.state.value.section && props.actions.state.value.content.length) props.actions.focus();
  else if (props.actions.state.value.canOpen) props.actions.open();
  else props.actions.inspect();
}
</script>

<template>
  <div
    ref="bar"
    class="w-max"
    :class="ready ? 'visible' : 'invisible'"
    :style="{ maxWidth: `${availableWidth}px` }"
  >
    <ToolbarRoot
      :class="CANVAS_ACTION_BAR_CLASSES"
      orientation="horizontal"
      :loop="false"
      :aria-label="t('spaces.canvas.nodeActions.label')"
    >
      <UiTooltip
        v-if="showPrimary && canvasBarPlacement('node-primary', minimal) === 'bar'"
        :text="primaryLabel"
        :disabled="!compact"
        side="top"
      >
        <ToolbarButton as-child>
          <button
            type="button"
            :class="[CANVAS_ACTION_BUTTON_CLASSES, 'font-medium text-foreground']"
            :aria-label="primaryLabel"
            @click="runPrimaryAction"
          >
            <component :is="primaryIcon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
            <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{ primaryLabel }}</span>
          </button>
        </ToolbarButton>
      </UiTooltip>
      <SpaceCanvasTextFormatBar
        v-if="textKind"
        :kind="textKind"
        :compact="compact"
        v-bind="actions.selectedAppearance.value ? { appearance: actions.selectedAppearance.value } : {}"
        @update="actions.setAppearance"
      />
      <UiTooltip
        v-if="actions.selectedVideoNode.value && canvasBarPlacement('video-edit', minimal) === 'bar'"
        :text="t('spaces.media.editVideo')"
        :disabled="!compact"
        side="top"
      >
        <ToolbarButton as-child>
          <button
            type="button"
            :class="CANVAS_ACTION_BUTTON_CLASSES"
            :aria-label="t('spaces.media.editVideo')"
            @click="actions.editVideo"
          >
            <Scissors class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
            <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{
              t("spaces.media.editVideo")
            }}</span>
          </button>
        </ToolbarButton>
      </UiTooltip>
      <UiSeparator
        v-if="
          (showPrimary && canvasBarPlacement('node-primary', minimal) === 'bar') ||
          textKind ||
          (actions.selectedVideoNode.value && canvasBarPlacement('video-edit', minimal) === 'bar')
        "
        orientation="vertical"
        class="mx-0.5 h-4"
      />
      <SpaceCanvasSelectionMenu
        :actions="actions"
        :derivations="derivations"
        node-bar
        :node-tools-visible="
          canvasBarPlacement('node-primary', minimal) === 'menu' &&
          ((showPrimary && !actions.state.value.section) || Boolean(actions.selectedVideoNode.value))
        "
        @invoke-derivation="emit('invokeDerivation', $event)"
      >
        <template #node-tools>
          <UiDropdownMenuItem
            v-if="
              showPrimary &&
              !actions.state.value.section &&
              canvasBarPlacement('node-primary', minimal) === 'menu'
            "
            @select="runPrimaryAction"
          >
            <component :is="primaryIcon" class="size-(--knovum-icon-size-sm)" />{{ primaryLabel }}
          </UiDropdownMenuItem>
          <UiDropdownMenuItem
            v-if="actions.selectedVideoNode.value && canvasBarPlacement('video-edit', minimal) === 'menu'"
            @select="actions.editVideo"
          >
            <Scissors class="size-(--knovum-icon-size-sm)" />{{ t("spaces.media.editVideo") }}
          </UiDropdownMenuItem>
        </template>
      </SpaceCanvasSelectionMenu>
      <SpaceNodeBarCommonActions @inspect="actions.inspect" @download="actions.download" />
    </ToolbarRoot>
  </div>
</template>
