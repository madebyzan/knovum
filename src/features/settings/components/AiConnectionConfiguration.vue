<script setup lang="ts">
import { ChevronDown, ChevronRight, Server, Settings, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import type { ProviderDescriptor } from "../../../shared/contracts/generated/ProviderDescriptor";
import type { RoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import { PROVIDER_LOGOS } from "../connections/provider-logos";
import AiConnectionModels from "./AiConnectionModels.vue";

defineProps<{
  connection: AiConnectionSettings;
  provider: ProviderDescriptor;
  providers: readonly ProviderDescriptor[];
  expanded: boolean;
  busy: boolean;
  protected: boolean;
  protectedModelTargets: readonly string[];
  createModelRequest?: { purpose: RoutingPurpose } | undefined;
}>();
const emit = defineEmits<{ toggle: []; edit: []; remove: [] }>();
const { t } = useI18n();
</script>

<template>
  <div class="overflow-hidden rounded-panel border border-stroke-default bg-surface-raised">
    <div class="flex min-h-16 items-center gap-3 px-4 py-3 sm:px-5">
      <button
        type="button"
        class="flex min-w-0 flex-1 items-center gap-3 rounded-md text-left focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :aria-expanded="expanded"
        :aria-controls="`connection-models-${connection.id}`"
        @click="emit('toggle')"
      >
        <span
          aria-hidden="true"
          class="flex size-10 shrink-0 items-center justify-center rounded-lg bg-surface-panel"
        >
          <img
            v-if="PROVIDER_LOGOS[connection.provider]"
            :src="PROVIDER_LOGOS[connection.provider]?.src"
            alt=""
            class="size-5 object-contain"
            :class="PROVIDER_LOGOS[connection.provider]?.monochrome && 'dark:invert'"
          />
          <Server v-else class="size-5 text-fg-secondary" />
        </span>
        <span class="min-w-0 flex-1"
          ><span class="block truncate text-body-lg font-semibold text-fg-primary">{{
            provider.displayName
          }}</span
          ><span class="block truncate text-caption text-fg-secondary"
            >{{ connection.name }} ·
            {{ t("settings.ai.modelCount", { count: connection.models.length }) }}</span
          ></span
        >
        <ChevronDown v-if="expanded" aria-hidden="true" class="size-4 shrink-0 text-fg-secondary" />
        <ChevronRight v-else aria-hidden="true" class="size-4 shrink-0 text-fg-secondary" />
      </button>
      <span aria-hidden="true" class="mx-1 h-5 w-px bg-stroke-divider" />
      <UiIconButton :label="t('common.settings')" size="sm" :disabled="busy" @click="emit('edit')"
        ><Settings class="size-4"
      /></UiIconButton>
      <UiIconButton
        :label="t('settings.ai.deleteConnection')"
        size="sm"
        class="hover:text-fg-danger"
        :disabled="busy || protected"
        @click="emit('remove')"
        ><Trash2 class="size-4"
      /></UiIconButton>
    </div>
    <div
      :id="`connection-models-${connection.id}`"
      class="grid transition-[grid-template-rows] duration-200 ease-standard motion-reduce:transition-none"
      :class="expanded ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
      :inert="!expanded"
      :aria-hidden="!expanded"
    >
      <div class="min-h-0 overflow-hidden">
        <AiConnectionModels
          :connection="connection"
          :providers="providers"
          :protected-model-targets="protectedModelTargets"
          :create-request="createModelRequest"
        />
      </div>
    </div>
  </div>
</template>
