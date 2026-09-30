<script setup lang="ts">
import { Bot, Check, ChevronDown, Monitor, Settings2 } from "@lucide/vue";
import {
  DropdownMenuGroup,
  DropdownMenuItemIndicator,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
} from "reka-ui";
import { computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import { availableModelTargets, defaultExecutionTarget } from "../../../../foundation/ai/ai-execution";
import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import { useExecutionCatalogQuery } from "../../../../foundation/ai/execution-catalog-query";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import { conversationTargetKey, type ConversationTarget } from "../../conversation/conversation-target";

const target = defineModel<ConversationTarget | undefined>();
const available = defineModel<boolean>("available", { default: false });
const { t } = useI18n();
const router = useRouter();
const settings = useAiSettingsStore();
const catalog = useExecutionCatalogQuery({ detectInstallations: false });

const models = computed(() => availableModelTargets(settings.settings, "knowledge"));
const modelChoices = computed(() =>
  models.value.map((option) => {
    const selected = option.target;
    const connection = settings.settings.connections.find(({ id }) => id === selected.connectionId);
    return {
      ...option,
      name: connection?.models.find(({ id }) => id === selected.modelId)?.name,
      connectionName: connection?.name,
    };
  }),
);
const desktopAgents = computed(() =>
  (catalog.data.value?.agents ?? []).filter((agent) =>
    settings.settings.desktopAgents.some((configured) => configured.kind === agent.kind),
  ),
);
const selectedKey = computed(() => (target.value ? conversationTargetKey(target.value) : ""));
const selectedLabel = computed(() => {
  const model = models.value.find(({ value }) => value === selectedKey.value);
  if (model) return model.label;
  const agent = desktopAgents.value.find(
    ({ kind }) => conversationTargetKey({ type: "agent", runtimeKind: kind }) === selectedKey.value,
  );
  if (agent) return agent.displayName;
  return t("spaces.assistant.agents.select");
});
watch(
  [selectedKey, models, desktopAgents, () => settings.ready],
  () => {
    available.value = Boolean(
      target.value &&
      settings.ready &&
      (models.value.some(({ value }) => value === selectedKey.value) ||
        desktopAgents.value.some(
          ({ kind }) => conversationTargetKey({ type: "agent", runtimeKind: kind }) === selectedKey.value,
        )),
    );
  },
  { immediate: true },
);
watch(
  [models, desktopAgents],
  () => {
    if (target.value) return;
    const preferred = defaultExecutionTarget(settings.settings, "knowledge");
    target.value = preferred ?? undefined;
  },
  { immediate: true },
);

function select(selected: ConversationTarget) {
  target.value = selected;
  settings.settings.defaults.defaultTarget = { ...selected };
}

onMounted(() => void settings.ensureLoaded());
</script>

<template>
  <div class="min-w-0 flex-1">
    <p v-if="settings.loadError" class="text-caption text-fg-danger" role="alert">
      {{ t("ai.settingsUnavailable") }}: {{ String(settings.loadError) }}
    </p>
    <UiDropdownMenu
      side="top"
      align="start"
      class="max-h-96 w-64 max-w-[calc(100vw-24px)] overflow-y-auto p-1.5"
    >
      <template #trigger>
        <UiButton
          variant="ghost"
          size="sm"
          class="max-w-full min-w-0 justify-start px-1.5 text-fg-secondary"
          :disabled="!settings.ready"
        >
          <Monitor v-if="target?.type === 'agent'" class="size-3 shrink-0" />
          <Bot v-else class="size-3 shrink-0" />
          <span class="min-w-0 truncate">{{ selectedLabel }}</span>
          <ChevronDown class="size-3 shrink-0" />
        </UiButton>
      </template>
      <DropdownMenuRadioGroup :model-value="selectedKey">
        <DropdownMenuGroup>
          <DropdownMenuLabel class="px-2.5 pt-1.5 pb-1 text-caption-sm font-medium text-fg-tertiary">
            {{ t("spaces.assistant.agents.knovumAssistant") }}
          </DropdownMenuLabel>
          <DropdownMenuRadioItem
            v-for="model in modelChoices"
            :key="model.value"
            :value="model.value"
            :text-value="model.label"
            class="relative flex min-h-12 cursor-default items-center rounded-md px-2.5 py-1.5 pr-8 outline-none data-highlighted:bg-surface-control-hover"
            @select="select(model.target)"
          >
            <span class="min-w-0 flex-1">
              <span class="block truncate text-label text-fg-primary">{{ model.name }}</span>
              <span class="block truncate text-caption-sm text-fg-tertiary">{{ model.connectionName }}</span>
            </span>
            <DropdownMenuItemIndicator class="absolute right-2.5 text-fg-accent"
              ><Check class="size-4"
            /></DropdownMenuItemIndicator>
          </DropdownMenuRadioItem>
          <p v-if="!models.length" class="px-2.5 py-2 text-caption text-fg-tertiary">
            {{ t("spaces.assistant.agents.noModel") }}
          </p>
        </DropdownMenuGroup>
        <UiSeparator class="my-1.5" />
        <DropdownMenuGroup>
          <DropdownMenuLabel class="px-2.5 pt-1.5 pb-1 text-caption-sm font-medium text-fg-tertiary">
            {{ t("spaces.assistant.agents.desktop") }}
          </DropdownMenuLabel>
          <DropdownMenuRadioItem
            v-for="agent in desktopAgents"
            :key="agent.kind"
            :value="conversationTargetKey({ type: 'agent', runtimeKind: agent.kind })"
            :text-value="agent.displayName"
            class="relative flex min-h-interaction cursor-default items-center rounded-md px-2.5 py-1.5 pr-8 outline-none data-highlighted:bg-surface-control-hover"
            @select="select({ type: 'agent', runtimeKind: agent.kind })"
          >
            <span class="min-w-0 flex-1 truncate text-label text-fg-primary">{{ agent.displayName }}</span>
            <DropdownMenuItemIndicator class="absolute right-2.5 text-fg-accent"
              ><Check class="size-4"
            /></DropdownMenuItemIndicator>
          </DropdownMenuRadioItem>
          <p v-if="catalog.error.value" class="px-2.5 py-2 text-caption text-fg-danger" role="alert">
            {{ String(catalog.error.value) }}
          </p>
        </DropdownMenuGroup>
      </DropdownMenuRadioGroup>
      <UiSeparator class="my-1.5" />
      <UiDropdownMenuItem
        class="gap-2.5 px-2.5 text-fg-secondary"
        @select="router.push({ name: 'settings', query: { pane: 'aiModels' } })"
      >
        <Settings2 class="size-4" />{{ t("spaces.assistant.agents.manage") }}
      </UiDropdownMenuItem>
    </UiDropdownMenu>
  </div>
</template>
