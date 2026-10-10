<script setup lang="ts">
import { Puzzle } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import type { AgentExtensionAvailability } from "../../../../shared/contracts/generated/AgentExtensionAvailability";
import type { AgentExtensionSelection } from "../../../../shared/contracts/generated/AgentExtensionSelection";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import ConversationMcpSelection from "./ConversationMcpSelection.vue";
const selection = defineModel<AgentExtensionSelection>({ required: true });
const props = defineProps<{
  availability?: AgentExtensionAvailability | undefined;
  disabled: boolean;
  error?: string | undefined;
}>();
const router = useRouter();
const { t } = useI18n();
const count = computed(
  () =>
    selection.value.pluginIds.length + selection.value.skillIds.length + selection.value.connections.length,
);
const hasUnavailableSelection = computed(
  () =>
    props.availability &&
    (selection.value.pluginIds.some(
      (id) => !props.availability?.plugins.some((plugin) => plugin.id === id),
    ) ||
      selection.value.skillIds.some((id) => !props.availability?.skills.some((skill) => skill.id === id)) ||
      selection.value.connections.some(
        (selected) =>
          !props.availability?.connections.some((connection) => connection.id === selected.connectionId),
      )),
);
function toggleExtension(kind: "pluginIds" | "skillIds", extensionId: string) {
  const previous = selection.value[kind];
  selection.value = {
    ...selection.value,
    [kind]: previous.includes(extensionId)
      ? previous.filter((value) => value !== extensionId)
      : [...previous, extensionId],
  };
}
const extensionGroups = computed(() =>
  [
    {
      kind: "pluginIds" as const,
      label: "plugins",
      items:
        props.availability?.plugins.map((plugin) => {
          const manifest = plugin.revision.manifest;
          const isBundled = plugin.source.type === "bundled";
          return {
            id: plugin.id,
            name: isBundled ? t(`extensions.bundled.${manifest.name}.name`) : manifest.name,
            description: isBundled
              ? t(`extensions.bundled.${manifest.name}.description`)
              : manifest.description,
          };
        }) ?? [],
    },
    { kind: "skillIds" as const, label: "skills", items: props.availability?.skills ?? [] },
  ].filter((group) => group.items.length),
);
</script>

<template>
  <UiPopover side="top" align="end" class="w-80 max-w-[calc(100vw-24px)] p-0">
    <template #trigger>
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="disabled"
        :aria-label="$t('spaces.conversation.extensions.label')"
        :title="$t('spaces.conversation.extensions.label')"
        class="gap-1 px-2"
        data-testid="conversation-extension-picker"
      >
        <Puzzle class="size-3.5" />
        <span v-if="count" class="text-caption tabular-nums">{{ count }}</span>
      </UiButton>
    </template>
    <header class="px-3 py-2.5 text-label font-medium text-fg-primary">
      {{ $t("spaces.conversation.extensions.label") }}
    </header>
    <UiScrollArea viewport-class="max-h-72">
      <section class="space-y-3 px-3 pb-3" :aria-label="$t('spaces.conversation.extensions.label')">
        <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
        <p v-if="hasUnavailableSelection" role="alert" class="text-caption text-fg-warning">
          {{ $t("spaces.conversation.extensions.unavailable") }}
          <UiButton
            size="sm"
            variant="ghost"
            :disabled="disabled"
            data-testid="clear-conversation-extensions"
            @click="selection = { pluginIds: [], skillIds: [], connections: [] }"
            >{{ $t("spaces.conversation.extensions.clearSelection") }}</UiButton
          >
        </p>

        <section v-for="group in extensionGroups" :key="group.kind" class="space-y-1">
          <h3 class="pb-1 text-caption font-medium text-fg-tertiary">
            {{ $t(`extensions.overview.${group.label}`) }}
          </h3>
          <label
            v-for="extension in group.items"
            :key="extension.id"
            class="flex cursor-default items-start gap-2.5 rounded-md p-2 hover:bg-surface-control-hover"
          >
            <UiCheckbox
              class="mt-0.5"
              :disabled="disabled"
              :model-value="selection[group.kind].includes(extension.id)"
              :label="extension.name"
              :data-testid="`conversation-${group.label}-${extension.id}`"
              @update:model-value="toggleExtension(group.kind, extension.id)"
            />
            <span class="min-w-0 flex-1">
              <span class="block text-label font-medium break-words text-fg-primary">{{
                extension.name
              }}</span>
              <span
                class="mt-0.5 line-clamp-2 text-caption break-words text-fg-secondary"
                :title="extension.description ?? undefined"
                >{{ extension.description }}</span
              >
            </span>
          </label>
        </section>
        <ConversationMcpSelection
          v-if="availability"
          v-model="selection"
          :connections="availability.connections"
          :disabled="disabled"
        />
        <p
          v-if="
            availability &&
            !availability.plugins.length &&
            !availability.skills.length &&
            !availability.connections.length
          "
          class="py-2 text-caption text-fg-secondary"
        >
          {{ $t("spaces.conversation.extensions.empty") }}
        </p>
        <p
          v-for="diagnostic in availability?.diagnostics"
          :key="diagnostic"
          class="text-caption text-fg-warning"
        >
          {{ diagnostic }}
        </p>
      </section>
    </UiScrollArea>
    <footer class="border-t border-stroke-divider p-1.5">
      <UiButton
        size="sm"
        variant="ghost"
        class="w-full justify-start"
        @click="router.push({ path: '/settings', query: { pane: 'extensions' } })"
        >{{ $t("spaces.conversation.extensions.manage") }}</UiButton
      >
    </footer>
  </UiPopover>
</template>
