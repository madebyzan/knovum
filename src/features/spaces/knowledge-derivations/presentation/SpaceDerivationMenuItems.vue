<script setup lang="ts">
import {
  Captions,
  ChevronRight,
  FileOutput,
  FilePenLine,
  Image,
  Layers3,
  ScanLine,
  Sparkles,
} from "@lucide/vue";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { computed, type Component } from "vue";
import { useI18n } from "vue-i18n";

import { canRunAction, shouldShowAction } from "../../../../shared/contracts/action-availability";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { spaceActionLabel } from "../catalog/action-label";
import type { CreationIntentIcon, DerivationUnavailableReason } from "../catalog/creation-intent";
import {
  CREATION_INTENT_DEFINITIONS,
  type CreationIntentId,
  type CreationIntentDecisions,
} from "../catalog/creation-intent-definitions";

const props = defineProps<{
  derivations: CreationIntentDecisions;
  rowClass: string;
  submenuClass: string;
  exclude?: readonly CreationIntentId[];
}>();
const emit = defineEmits<{ invoke: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const ICONS = {
  captions: Captions,
  "file-output": FileOutput,
  "file-pen": FilePenLine,
  image: Image,
  layers: Layers3,
  scan: ScanLine,
  sparkles: Sparkles,
} satisfies Record<CreationIntentIcon, Component>;
const derivationActions = computed(() =>
  CREATION_INTENT_DEFINITIONS.filter(({ id }) => {
    if (props.exclude?.includes(id)) return false;
    const decision = props.derivations[id];
    return shouldShowAction(decision);
  }),
);
const hasActions = computed(() => derivationActions.value.length > 0);

function reasonLabel(reason: DerivationUnavailableReason | undefined) {
  if (reason === "active-node-operation") return t("spaces.creation.shared.finishNodeOperation");
  if (reason === "needs-document-recognition")
    return t("spaces.creation.document.conversion.recognitionRequired");
  if (reason === "too-many-video-images") return t("spaces.creation.video.generation.tooManyImages");
  if (reason === "needs-indexed-knowledge") return t("spaces.creation.shared.needsIndexedKnowledge");
  return undefined;
}

function actionReason(actionId: CreationIntentId) {
  const availability = props.derivations[actionId];
  return availability.state === "blocked" ? (reasonLabel(availability.reason) ?? "") : "";
}

function selectAction(event: Event, actionId: CreationIntentId) {
  if (!canRunAction(props.derivations[actionId])) {
    event.preventDefault();
    return;
  }
  emit("invoke", actionId);
}
</script>

<template>
  <DropdownMenuSub v-if="hasActions">
    <DropdownMenuSubTrigger :class="props.rowClass">
      <Sparkles class="size-3.5" />
      <span class="min-w-0 flex-1">{{ t("spaces.creation.shared.createFromSelection") }}</span>
      <ChevronRight class="size-3 text-fg-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="props.submenuClass">
        <UiTooltip
          v-for="action in derivationActions"
          :key="action.id"
          :text="actionReason(action.id)"
          :disabled="canRunAction(props.derivations[action.id])"
          side="right"
        >
          <UiDropdownMenuItem
            :class="props.rowClass"
            :aria-disabled="!canRunAction(props.derivations[action.id])"
            @select="selectAction($event, action.id)"
          >
            <component :is="ICONS[action.icon]" class="size-3.5" />{{ spaceActionLabel(action.id) }}
          </UiDropdownMenuItem>
        </UiTooltip>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
</template>
