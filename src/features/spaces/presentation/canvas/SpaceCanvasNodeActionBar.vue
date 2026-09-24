<script setup lang="ts">
import {
  ArrowUpRight,
  Code2,
  ExternalLink,
  FileText,
  Focus,
  Image as ImageIcon,
  Info,
  Pencil,
  Play,
  Scissors,
  Volume2,
} from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed, toRef, useTemplateRef, type Component } from "vue";
import { useI18n } from "vue-i18n";

import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodeKind } from "../../model/nodes/node-payload";
import { CANVAS_ACTION_BAR_CLASSES, CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
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
const { compact } = useNodeActionBarDensity(bar, toRef(props, "availableWidth"), densityIdentity);
const openLabels = {
  note: "common.edit",
  text: "spaces.navigation.inspect",
  markdown: "common.edit",
  subtitle: "spaces.navigation.inspect",
  website: "spaces.canvas.nodeActions.openOriginal",
  portal: "spaces.canvas.nodeActions.goToTarget",
  image: "spaces.canvas.nodeActions.viewLarge",
  video: "spaces.canvas.nodeActions.openVideo",
  audio: "spaces.canvas.nodeActions.openAudio",
  file: "spaces.canvas.nodeActions.openFile",
  code: "spaces.canvas.nodeActions.viewCode",
  credential: "spaces.navigation.inspect",
  object: "spaces.navigation.inspect",
} satisfies Record<SpaceNodeKind, string>;
const openIcons = {
  note: Pencil,
  text: Info,
  markdown: Pencil,
  subtitle: Info,
  website: ExternalLink,
  portal: ArrowUpRight,
  image: ImageIcon,
  video: Play,
  audio: Volume2,
  file: FileText,
  code: Code2,
  credential: Info,
  object: Info,
} satisfies Record<SpaceNodeKind, Component>;
const primaryLabel = computed(() => {
  if (props.actions.state.value.section && props.actions.state.value.content.length)
    return t("spaces.navigation.focusSelection");
  const kind = props.node?.data.kind;
  return t(kind && props.actions.state.value.canOpen ? openLabels[kind] : "spaces.navigation.inspect");
});
const primaryIcon = computed(() => {
  if (props.actions.state.value.section && props.actions.state.value.content.length) return Focus;
  const kind = props.node?.data.kind;
  return kind && props.actions.state.value.canOpen ? openIcons[kind] : Info;
});
const showPrimary = computed(() => {
  if (props.actions.state.value.section && props.actions.state.value.content.length) return true;
  const kind = props.node?.data.kind;
  return Boolean(
    kind && props.actions.state.value.canOpen && openLabels[kind] !== "spaces.navigation.inspect",
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
  <div ref="bar" class="w-max" :style="{ maxWidth: `${availableWidth}px` }">
    <ToolbarRoot
      :class="CANVAS_ACTION_BAR_CLASSES"
      orientation="horizontal"
      :loop="false"
      :aria-label="t('spaces.canvas.nodeActions.label')"
    >
      <ToolbarButton v-if="showPrimary" as-child>
        <button
          type="button"
          :class="[CANVAS_ACTION_BUTTON_CLASSES, 'font-medium text-foreground']"
          :aria-label="primaryLabel"
          :title="compact ? primaryLabel : undefined"
          @click="runPrimaryAction"
        >
          <component :is="primaryIcon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
          <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{ primaryLabel }}</span>
        </button>
      </ToolbarButton>
      <SpaceCanvasTextFormatBar
        v-if="textKind"
        :kind="textKind"
        v-bind="actions.selectedAppearance.value ? { appearance: actions.selectedAppearance.value } : {}"
        @update="actions.setAppearance"
      />
      <ToolbarButton v-if="actions.selectedVideoNode.value" as-child>
        <button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :aria-label="t('spaces.media.editVideo')"
          :title="compact ? t('spaces.media.editVideo') : undefined"
          @click="actions.editVideo"
        >
          <Scissors class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
          <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{
            t("spaces.media.editVideo")
          }}</span>
        </button>
      </ToolbarButton>
      <UiSeparator
        v-if="showPrimary || textKind || actions.selectedVideoNode.value"
        orientation="vertical"
        class="mx-0.5 h-4"
      />
      <SpaceCanvasSelectionMenu
        :actions="actions"
        :derivations="derivations"
        node-bar
        @invoke-derivation="emit('invokeDerivation', $event)"
      />
      <SpaceNodeBarCommonActions @inspect="actions.inspect" @download="actions.download" />
    </ToolbarRoot>
  </div>
</template>
