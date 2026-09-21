<script setup lang="ts">
import { Captions, ChevronRight, FileOutput, FilePenLine, Image, ScanLine, Sparkles } from "@lucide/vue";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { computed, type Component } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../shared/ui/UiDropdownMenuItem.vue";
import type {
  CreationIntentIcon,
  DerivationUnavailableReason,
} from "../knowledge-derivations/catalog/creation-intent-contract";
import {
  creationIntentDefinitions,
  type CreationIntentId,
  type CreationIntentDecisions,
} from "../knowledge-derivations/catalog/creation-intent-definitions";

const props = defineProps<{
  derivations: CreationIntentDecisions;
  rowClass: string;
  submenuClass: string;
}>();
const emit = defineEmits<{ invoke: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const icons = {
  captions: Captions,
  "file-output": FileOutput,
  "file-pen": FilePenLine,
  image: Image,
  scan: ScanLine,
  sparkles: Sparkles,
} satisfies Record<CreationIntentIcon, Component>;
const derivationActions = computed(() =>
  creationIntentDefinitions.filter(({ id }) => {
    const decision = props.derivations[id];
    return decision.available || decision.reason;
  }),
);
const hasActions = computed(() => derivationActions.value.length > 0);

function reasonLabel(reason: DerivationUnavailableReason | undefined) {
  if (reason === "too-many-video-images") return t("spaces.creation.tooManyVideoImages");
  if (reason === "needs-indexed-knowledge") return t("spaces.creation.needsIndexedKnowledge");
  return undefined;
}
</script>

<template>
  <DropdownMenuSub v-if="hasActions">
    <DropdownMenuSubTrigger :class="props.rowClass">
      <Sparkles class="size-(--knovum-icon-size-sm)" />
      <span class="min-w-0 flex-1">{{ t("spaces.creation.createFromSelection") }}</span>
      <ChevronRight class="size-(--knovum-icon-size-xs) text-foreground-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="props.submenuClass">
        <UiDropdownMenuItem
          v-for="action in derivationActions"
          :key="action.id"
          :class="props.rowClass"
          :disabled="!props.derivations[action.id].available"
          :title="reasonLabel(props.derivations[action.id].reason)"
          @select="emit('invoke', action.id)"
        >
          <component :is="icons[action.icon]" class="size-(--knovum-icon-size-sm)" />{{ t(action.labelKey) }}
        </UiDropdownMenuItem>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
</template>
