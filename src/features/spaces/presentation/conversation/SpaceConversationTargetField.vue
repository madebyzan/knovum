<script setup lang="ts">
import { Bot, ChevronDown, Monitor, Settings2 } from "@lucide/vue";
import { computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import { availableExecutionTargets, defaultExecutionTarget } from "../../../../foundation/ai/ai-execution";
import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import { useExecutionCatalogQuery } from "../../../../foundation/ai/execution-catalog-query";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDropdownMenu, { type UiDropdownMenuItemOption } from "../../../../shared/ui/UiDropdownMenu.vue";
import { conversationTargetKey, type ConversationTarget } from "../../conversation/conversation-target";

const target = defineModel<ConversationTarget | undefined>();
const available = defineModel<boolean>("available", { default: false });
const { t } = useI18n();
const router = useRouter();
const settings = useAiSettingsStore();
const catalog = useExecutionCatalogQuery({ detectInstallations: false });

const models = computed(() => availableExecutionTargets(settings.settings, "knowledge"));
const desktopAgents = computed(() =>
  (catalog.data.value?.agents ?? []).filter((agent) =>
    settings.settings.desktopAgents.some((configured) => configured.kind === agent.kind),
  ),
);
const selectedKey = computed(() => (target.value ? conversationTargetKey(target.value) : ""));
const selectedLabel = computed(() => {
  const model = models.value.find(({ value }) => value === selectedKey.value);
  if (model) return `${t("spaces.assistant.agents.builtIn")} · ${model.label}`;
  const agent = desktopAgents.value.find(({ kind }) => `agent:${kind}` === selectedKey.value);
  if (agent) return `${t("spaces.assistant.agents.external")} · ${agent.displayName}`;
  return t("spaces.assistant.agents.select");
});
const choices = computed<UiDropdownMenuItemOption[]>(() => [
  { label: t("spaces.assistant.agents.builtIn"), disabled: true },
  ...models.value.map((model) => ({
    id: model.value,
    label: model.label,
    icon: Bot,
    checked: selectedKey.value === model.value,
    onSelect: () => (target.value = model.target),
  })),
  ...(!models.value.length ? [{ label: t("spaces.assistant.agents.noModel"), disabled: true }] : []),
  {
    label: t("ai.openSettings"),
    icon: Settings2,
    onSelect: () => void router.push({ name: "settings", query: { pane: "aiModels" } }),
  },
  { separator: true },
  { label: t("spaces.assistant.agents.external"), disabled: true },
  ...desktopAgents.value.map((agent) => ({
    id: `agent:${agent.kind}`,
    label: agent.displayName,
    icon: Monitor,
    checked: selectedKey.value === `agent:${agent.kind}`,
    onSelect: () => (target.value = { type: "agent", runtimeKind: agent.kind }),
  })),
  ...(catalog.error.value ? [{ label: String(catalog.error.value), disabled: true }] : []),
  {
    label: t("spaces.assistant.agents.configure"),
    icon: Settings2,
    onSelect: () => void router.push({ name: "settings", query: { pane: "aiModels" } }),
  },
]);

watch(
  [selectedKey, models, desktopAgents, () => settings.ready],
  () => {
    available.value = Boolean(
      target.value &&
      settings.ready &&
      (models.value.some(({ value }) => value === selectedKey.value) ||
        desktopAgents.value.some(({ kind }) => `agent:${kind}` === selectedKey.value)),
    );
  },
  { immediate: true },
);
watch(
  [models, desktopAgents],
  () => {
    if (target.value) return;
    const preferred = defaultExecutionTarget(settings.settings, "knowledge");
    target.value =
      models.value.find((model) => preferred && model.value === conversationTargetKey(preferred))?.target ??
      models.value[0]?.target ??
      (desktopAgents.value[0] ? { type: "agent", runtimeKind: desktopAgents.value[0].kind } : undefined);
  },
  { immediate: true },
);

onMounted(() => void settings.ensureLoaded());
</script>

<template>
  <div class="min-w-0 flex-1">
    <p v-if="settings.loadError" class="text-caption text-fg-danger" role="alert">
      {{ t("ai.settingsUnavailable") }}: {{ String(settings.loadError) }}
    </p>
    <UiDropdownMenu :items="choices" side="top" align="start" class="max-h-72 max-w-80 overflow-y-auto">
      <template #trigger>
        <UiButton
          variant="ghost"
          size="sm"
          class="max-w-full min-w-0 justify-start px-1.5 text-fg-secondary"
          :disabled="!settings.ready"
        >
          <Bot v-if="target?.type === 'model'" class="size-3 shrink-0" />
          <Monitor v-else class="size-3 shrink-0" />
          <span class="min-w-0 truncate">{{ selectedLabel }}</span>
          <ChevronDown class="size-3 shrink-0" />
        </UiButton>
      </template>
    </UiDropdownMenu>
  </div>
</template>
