<script setup lang="ts">
import { Captions, Clapperboard, FilePenLine, Image, Layers3, ScanLine, Sticker } from "@lucide/vue";
import { computed, type Component } from "vue";
import { useI18n } from "vue-i18n";

import { canRunAction } from "../../../../shared/contracts/action-availability";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { spaceActionLabel } from "../catalog/action-label";
import type { CreationIntentIcon, DerivationUnavailableReason } from "../catalog/creation-intent";
import type { CreationIntentDecisions, CreationIntentId } from "../catalog/creation-intent-definitions";
import { visibleCreationActions } from "./visible-derivation-actions";

const props = defineProps<{ derivations: CreationIntentDecisions }>();
const emit = defineEmits<{ invoke: [actionId: CreationIntentId] }>();
const { t } = useI18n();

const ICONS = {
  captions: Captions,
  clapperboard: Clapperboard,
  "file-pen": FilePenLine,
  image: Image,
  layers: Layers3,
  scan: ScanLine,
  sticker: Sticker,
} satisfies Record<CreationIntentIcon, Component>;

const creationActions = computed(() => visibleCreationActions(props.derivations));

function reasonLabel(reason: DerivationUnavailableReason | undefined) {
  if (reason === "active-node-operation") return t("spaces.creation.shared.finishNodeOperation");
  if (reason === "too-many-video-images") return t("spaces.creation.video.generation.tooManyImages");
  if (reason === "too-many-sources") return t("spaces.creation.shared.tooManySources");
  if (reason === "no-usable-source") return t("spaces.creation.shared.noUsableSource");
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
  <UiTooltip
    v-for="action in creationActions"
    :key="action.id"
    :text="actionReason(action.id)"
    :disabled="canRunAction(props.derivations[action.id])"
    side="right"
  >
    <UiDropdownMenuItem
      :aria-disabled="!canRunAction(props.derivations[action.id])"
      @select="selectAction($event, action.id)"
    >
      <component :is="ICONS[action.icon]" class="size-3.5" />{{ spaceActionLabel(action.id) }}
    </UiDropdownMenuItem>
  </UiTooltip>
</template>
