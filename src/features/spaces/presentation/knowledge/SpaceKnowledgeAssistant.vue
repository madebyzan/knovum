<script setup lang="ts">
import { shallowRef } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import SpaceActionCatalog from "../actions/SpaceActionCatalog.vue";
import SpaceBuiltInAssistant from "../assistant/SpaceBuiltInAssistant.vue";
import SpaceExternalAgentConversation from "../assistant/SpaceExternalAgentConversation.vue";
import SpaceAutomationCenter from "../automation/SpaceAutomationCenter.vue";
import SpaceRunCenter from "../runs/SpaceRunCenter.vue";
import SpaceExternalTools from "../workflows/SpaceExternalTools.vue";
import SpaceWorkflowStudio from "../workflows/SpaceWorkflowStudio.vue";
import SpaceGroundedAssistant from "./SpaceGroundedAssistant.vue";

type AssistantSurface =
  | "grounded"
  | "built-in"
  | "externalAgent"
  | "actions"
  | "workflows"
  | "runs"
  | "automation"
  | "externalTools";
const surface = shallowRef<AssistantSurface>("built-in");
</script>

<template>
  <section class="flex h-full min-h-0 flex-col bg-panel">
    <nav
      class="flex shrink-0 gap-1 overflow-x-auto border-b border-divider p-2"
      :aria-label="$t('spaces.assistant.surfaceLabel')"
    >
      <UiButton
        size="sm"
        :variant="surface === 'built-in' ? 'secondary' : 'ghost'"
        @click="surface = 'built-in'"
      >
        {{ $t("spaces.assistant.builtIn") }}
      </UiButton>
      <UiButton
        size="sm"
        :variant="surface === 'grounded' ? 'secondary' : 'ghost'"
        @click="surface = 'grounded'"
      >
        {{ $t("spaces.assistant.grounded") }}
      </UiButton>
      <UiButton
        size="sm"
        :variant="surface === 'externalAgent' ? 'secondary' : 'ghost'"
        @click="surface = 'externalAgent'"
      >
        {{ $t("spaces.assistant.externalAgent") }}
      </UiButton>
      <UiButton
        size="sm"
        :variant="surface === 'actions' ? 'secondary' : 'ghost'"
        @click="surface = 'actions'"
      >
        {{ $t("spaces.assistant.actions") }}
      </UiButton>
      <UiButton
        size="sm"
        :variant="surface === 'workflows' ? 'secondary' : 'ghost'"
        @click="surface = 'workflows'"
      >
        {{ $t("spaces.assistant.workflows") }}
      </UiButton>
      <UiButton size="sm" :variant="surface === 'runs' ? 'secondary' : 'ghost'" @click="surface = 'runs'">
        {{ $t("spaces.assistant.runs") }}
      </UiButton>
      <UiButton
        size="sm"
        :variant="surface === 'automation' ? 'secondary' : 'ghost'"
        @click="surface = 'automation'"
      >
        {{ $t("spaces.assistant.automation") }}
      </UiButton>
      <UiButton
        size="sm"
        :variant="surface === 'externalTools' ? 'secondary' : 'ghost'"
        @click="surface = 'externalTools'"
      >
        {{ $t("spaces.assistant.externalTools") }}
      </UiButton>
    </nav>
    <SpaceBuiltInAssistant v-if="surface === 'built-in'" class="min-h-0 flex-1" />
    <SpaceGroundedAssistant v-else-if="surface === 'grounded'" class="min-h-0 flex-1" />
    <SpaceExternalAgentConversation v-else-if="surface === 'externalAgent'" class="min-h-0 flex-1" />
    <SpaceActionCatalog v-else-if="surface === 'actions'" class="min-h-0 flex-1" />
    <SpaceWorkflowStudio v-else-if="surface === 'workflows'" class="min-h-0 flex-1" />
    <SpaceRunCenter v-else-if="surface === 'runs'" class="min-h-0 flex-1" />
    <SpaceAutomationCenter v-else-if="surface === 'automation'" class="min-h-0 flex-1" />
    <SpaceExternalTools v-else class="min-h-0 flex-1" />
  </section>
</template>
