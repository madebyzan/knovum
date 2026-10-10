<script setup lang="ts">
import { ArrowLeft, FilePlus2, MoreHorizontal, Settings2, UserRoundPlus, X } from "@lucide/vue";
import { RouterLink } from "vue-router";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { NodeSpeechProjectType } from "../../speech/node-speech";
import { NODE_SPEECH_OPERATIONS } from "./node-speech-operations";

defineProps<{
  node: SpaceCanvasNode;
  projectType: NodeSpeechProjectType;
  revision?: number | undefined;
  isCreating: boolean;
  isAddingProject: boolean;
  isAddingVoice: boolean;
}>();
const emit = defineEmits<{ newProject: []; addVoice: []; close: [] }>();
</script>

<template>
  <header class="flex shrink-0 items-center gap-2 border-b border-stroke-divider p-3">
    <UiIconButton
      v-if="isAddingProject || isAddingVoice"
      size="sm"
      :label="$t('common.back')"
      tooltip
      @click="isAddingVoice ? emit('addVoice') : emit('newProject')"
    >
      <ArrowLeft class="size-3.5" />
    </UiIconButton>
    <component
      v-else
      :is="NODE_SPEECH_OPERATIONS[projectType].icon"
      class="size-4 shrink-0 text-fg-secondary"
      :stroke-width="1.6"
    />
    <div class="min-w-0 flex-1">
      <h2 class="text-title-sm font-semibold">
        {{ $t(NODE_SPEECH_OPERATIONS[projectType].label)
        }}<span v-if="revision" class="ml-1 text-caption-sm font-normal text-fg-tertiary"
          >v{{ revision }}</span
        >
      </h2>
      <p class="mt-0.5 truncate text-caption text-fg-secondary" :title="node.data.title">
        {{ node.data.title }}
      </p>
    </div>
    <UiDropdownMenu v-if="!isAddingProject && !isAddingVoice" :modal="false">
      <template #trigger
        ><UiIconButton size="sm" :label="$t('common.more')" :title="$t('common.more')"
          ><MoreHorizontal class="size-3.5" /></UiIconButton
      ></template>
      <UiDropdownMenuItem v-if="revision" :disabled="isCreating" @select="emit('newProject')">
        <FilePlus2 class="size-3.5" />{{ $t("spaces.speech.project.new") }}
      </UiDropdownMenuItem>
      <UiDropdownMenuItem as-child>
        <RouterLink :to="{ path: '/settings', query: { pane: 'voices' } }" data-testid="manage-voices"
          ><Settings2 class="size-3.5" />{{ $t("spaces.speech.library.collectionTitle") }}</RouterLink
        >
      </UiDropdownMenuItem>
      <UiDropdownMenuItem data-testid="add-reference-voice" @select="emit('addVoice')"
        ><UserRoundPlus class="size-3.5" />{{ $t("spaces.speech.library.addTitle") }}</UiDropdownMenuItem
      >
    </UiDropdownMenu>
    <UiIconButton :label="$t('common.close')" size="sm" tooltip @click="emit('close')"
      ><X class="size-3.5"
    /></UiIconButton>
  </header>
</template>
