<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { computed, reactive, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { formatBytes } from "../../../foundation/format/bytes";
import type { LocalModelAvailability } from "../../../shared/contracts/generated/LocalModelAvailability";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import LocalModelConfiguration from "./LocalModelConfiguration.vue";

const props = defineProps<{
  capability: string;
  models: readonly LocalModelAvailability[];
  visibleIds: ReadonlySet<string>;
  filtering: boolean;
}>();
const { t } = useI18n();
const expanded = shallowRef(false);
const preparingModels = reactive(new Set<string>());
const currentModel = computed(() => props.models.find((model) => model.selected || model.fixed));
const installedBytes = computed(() => props.models.reduce((bytes, model) => bytes + model.installedBytes, 0));
const visibleModels = computed(() => props.models.filter((model) => props.visibleIds.has(model.id)));
const open = computed(() => props.filtering || expanded.value);
const currentStatus = computed(() =>
  currentModel.value?.verified ? "ready" : currentModel.value?.installed ? "invalid" : "notInstalled",
);

function updatePreparation(modelId: string, active: boolean) {
  if (active) preparingModels.add(modelId);
  else preparingModels.delete(modelId);
}
</script>

<template>
  <UiSettingsSection :data-model-capability="capability">
    <CollapsibleRoot :open="open" @update:open="expanded = $event">
      <component
        :is="filtering ? 'div' : CollapsibleTrigger"
        :data-testid="`model-capability-${capability}`"
        class="flex min-h-(--knovum-setting-min-height) w-full items-center gap-3 px-4 py-3 text-left"
        :class="
          !filtering &&
          'hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-stroke-focus'
        "
      >
        <span class="min-w-0 flex-1">
          <span role="heading" aria-level="3" class="block text-body-lg font-medium text-fg-primary">
            {{ t(`settings.ai.localModels.categoryLabels.${capability}`) }}
          </span>
          <span class="mt-0.5 flex flex-wrap gap-x-2 gap-y-0.5 text-caption text-fg-secondary">
            <template v-if="currentModel">
              <span class="min-w-0 break-words">{{
                t(`settings.ai.localModels.${currentModel.fixed ? "fixedModel" : "configuredModel"}`, {
                  name: currentModel.name,
                })
              }}</span>
              <span :class="currentStatus === 'invalid' && 'text-fg-warning'">{{
                t(`settings.ai.localModels.${currentStatus}`)
              }}</span>
            </template>
            <span v-else>{{ t("settings.ai.localModels.modelCount", { count: models.length }) }}</span>
            <span v-if="preparingModels.size" role="status">{{
              t("settings.ai.localModels.preparationCount", { count: preparingModels.size })
            }}</span>
          </span>
        </span>
        <span
          v-if="installedBytes"
          class="max-w-28 shrink-0 text-right text-caption text-fg-tertiary tabular-nums"
          >{{ t("settings.ai.localModels.installedSize", { size: formatBytes(installedBytes) }) }}</span
        >
        <ChevronDown
          v-if="!filtering"
          class="size-4 shrink-0 text-fg-tertiary"
          :class="open && 'rotate-180'"
          aria-hidden="true"
        />
      </component>
      <CollapsibleContent v-show="open" force-mount>
        <p class="border-t border-stroke-divider px-4 pt-3 text-caption text-fg-secondary">
          {{ t(`settings.ai.localModels.capabilities.${capability}`) }}
        </p>
        <p v-if="capability === 'speechSynthesis'" class="px-4 pt-1 text-caption text-fg-secondary">
          {{ t("settings.ai.localModels.selectedVoiceModel") }}
        </p>
        <LocalModelConfiguration
          v-for="model in models"
          :key="model.id"
          v-show="visibleIds.has(model.id)"
          :model="model"
          :class="model.id !== visibleModels[0]?.id && 'border-t border-stroke-divider'"
          @preparation-change="updatePreparation(model.id, $event)"
          @failure="expanded = true"
        />
      </CollapsibleContent>
    </CollapsibleRoot>
  </UiSettingsSection>
</template>
