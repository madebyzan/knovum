<script setup lang="ts">
import { ChevronRight, RotateCcw } from "@lucide/vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { useI18n } from "vue-i18n";

import type { ModelChoiceOption } from "../../../../foundation/ai/model-choice";
import ModelChoiceTrigger from "../../../../foundation/ai/ModelChoiceTrigger.vue";
import type { LocalModelId } from "../../../../shared/contracts/generated/LocalModelId";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { EnrichmentJobState } from "../../contracts/generated/EnrichmentJobState";
import type { SpaceTranscriptEdit } from "../schema/knowledge";
import type { SpaceMediaTranscript } from "../schema/transcript";
import SpaceDiarizationControl from "./SpaceDiarizationControl.vue";
import SpaceTranscriptionLanguageSelect from "./SpaceTranscriptionLanguageSelect.vue";
import SpaceTranscriptNavigator from "./SpaceTranscriptNavigator.vue";

defineProps<{
  nodeId: string;
  transcript: SpaceMediaTranscript;
  transcriptionModels: readonly ModelChoiceOption[];
  diarizationJob?: EnrichmentJobState | undefined;
  currentMs?: number | undefined;
  transcribeLocked: boolean;
  deriveLocked: boolean;
  transcriptEditLocked: boolean;
}>();

const open = defineModel<boolean>("open", { required: true });
const language = defineModel<string>("language", { required: true });
const emit = defineEmits<{
  analyze: [modelId?: LocalModelId];
  seek: [startMs: number];
  saveSegment: [payload: SpaceTranscriptEdit];
  alignSegments: [segmentIds: string[], model?: LocalModelId];
}>();
const { t } = useI18n();
</script>

<template>
  <CollapsibleRoot v-model:open="open" as-child>
    <div class="mt-3.5">
      <div class="-mx-1.5 flex items-center gap-1">
        <CollapsibleTrigger
          class="flex min-h-interaction min-w-0 flex-1 items-center gap-1.5 rounded-md px-1.5 text-caption font-medium text-fg-secondary transition-colors hover:bg-surface-control-hover hover:text-fg-primary"
        >
          <ChevronRight
            class="size-3 shrink-0 text-fg-tertiary transition-transform duration-150 ease-standard"
            :class="open && 'rotate-90'"
          />
          <span>{{ t("spaces.transcript.text.title") }}</span>
          <span class="ml-auto text-caption-sm font-normal text-fg-tertiary tabular-nums">
            {{ transcript.language }} · {{ transcript.segments.length }}
          </span>
        </CollapsibleTrigger>
        <ModelChoiceTrigger
          :options="transcriptionModels"
          :disabled="transcribeLocked"
          @select="(id) => emit('analyze', id as LocalModelId)"
        >
          <template #trigger="{ multiple }">
            <UiIconButton
              :label="t('spaces.media.player.retranscribe')"
              tooltip
              size="sm"
              :disabled="transcribeLocked"
              @click="!multiple && emit('analyze')"
            >
              <RotateCcw class="size-3" />
            </UiIconButton>
          </template>
        </ModelChoiceTrigger>
      </div>
      <SpaceTranscriptionLanguageSelect v-model="language" class="mt-1.5" :disabled="transcribeLocked" />
      <CollapsibleContent class="mt-2">
        <SpaceDiarizationControl
          :node-id="nodeId"
          :revision="transcript.revision"
          :duration-ms="transcript.durationMs"
          :speaker-count="transcript.speakerTracks.length"
          :active-job="diarizationJob"
          :disabled="deriveLocked"
        />
        <SpaceTranscriptNavigator
          :node-id="nodeId"
          :segments="transcript.segments"
          v-bind="currentMs !== undefined ? { currentMs } : {}"
          :disabled="transcriptEditLocked"
          :alignment-disabled="deriveLocked"
          @seek="emit('seek', $event)"
          @save="emit('saveSegment', $event)"
          @align="(segmentIds, model) => emit('alignSegments', segmentIds, model)"
        />
      </CollapsibleContent>
    </div>
  </CollapsibleRoot>
</template>
