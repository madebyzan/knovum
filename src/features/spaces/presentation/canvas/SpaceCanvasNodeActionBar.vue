<script setup lang="ts">
import {
  ArrowUpRight,
  Code2,
  Copy,
  Download,
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
import { computed, type Component } from "vue";
import { useI18n } from "vue-i18n";

import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodeKind } from "../../model/nodes/node-payload";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceCanvasTextFormatBar from "./SpaceCanvasTextFormatBar.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  node?: SpaceCanvasNode | undefined;
}>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const commandClass =
  "inline-flex h-(--knovum-control-height-md) shrink-0 items-center gap-1.5 rounded-control px-2.5 text-label text-foreground-secondary outline-none transition-colors duration-100 hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring data-[state=open]:bg-control-hover";
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
  <ToolbarRoot
    class="flex h-(--knovum-control-height-lg) w-max cursor-default items-center gap-0.5 rounded-panel border border-border-strong bg-overlay p-px shadow-control select-none"
    orientation="horizontal"
    :loop="false"
    :aria-label="t('spaces.canvas.nodeActions.label')"
  >
    <ToolbarButton as-child>
      <button
        type="button"
        :class="[commandClass, 'font-medium text-foreground']"
        @click="runPrimaryAction"
      >
        <component :is="primaryIcon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
        {{ primaryLabel }}
      </button>
    </ToolbarButton>
    <UiSeparator v-if="actions.state.value.canOpen" orientation="vertical" class="mx-0.5 h-4" />
    <ToolbarButton v-if="actions.state.value.canOpen" as-child>
      <button type="button" :class="commandClass" @click="actions.inspect">
        <Info class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
        {{ t("spaces.navigation.inspect") }}
      </button>
    </ToolbarButton>
    <SpaceCanvasTextFormatBar
      v-if="textKind"
      :kind="textKind"
      v-bind="actions.selectedAppearance.value ? { appearance: actions.selectedAppearance.value } : {}"
      @update="actions.setAppearance"
    />
    <ToolbarButton v-if="actions.selectedVideoNode.value" as-child>
      <button type="button" :class="commandClass" @click="actions.editVideo">
        <Scissors class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
        {{ t("spaces.media.editVideo") }}
      </button>
    </ToolbarButton>
    <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
    <ToolbarButton as-child>
      <button type="button" :class="commandClass" @click="actions.duplicate">
        <Copy class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
        {{ t("spaces.navigation.duplicateSelection") }}
      </button>
    </ToolbarButton>
    <SpaceCanvasSelectionMenu
      :actions="actions"
      :derivations="derivations"
      node-bar
      duplicate-in-bar
      @invoke-derivation="emit('invokeDerivation', $event)"
    />
    <div class="flex h-full items-center border-l border-divider pl-0.5">
      <ToolbarButton as-child>
        <button type="button" :class="[commandClass, 'font-medium text-foreground']" @click="actions.download">
          <Download class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
          {{ t("spaces.canvas.nodeActions.download") }}
        </button>
      </ToolbarButton>
    </div>
  </ToolbarRoot>
</template>
