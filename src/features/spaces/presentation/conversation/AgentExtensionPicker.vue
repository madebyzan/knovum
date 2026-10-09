<script setup lang="ts">
import { Puzzle } from "@lucide/vue";
import { computed } from "vue";
import { useRouter } from "vue-router";

import type { AgentExtensionAvailability } from "../../../../shared/contracts/generated/AgentExtensionAvailability";
import type { AgentExtensionSelection } from "../../../../shared/contracts/generated/AgentExtensionSelection";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import ConversationConnectionSelection from "./ConversationConnectionSelection.vue";
const selection = defineModel<AgentExtensionSelection>({ required: true });
const props = defineProps<{
  availability?: AgentExtensionAvailability | undefined;
  disabled: boolean;
  error?: string | undefined;
}>();
const router = useRouter();
const count = computed(() => selection.value.skillIds.length + selection.value.connections.length);
const selectedConnections = computed(() =>
  props.availability?.connections.filter((connection) => isConnectionSelected(connection.id)),
);
function isConnectionSelected(connectionId: string) {
  return selection.value.connections.some((connection) => connection.connectionId === connectionId);
}
const unavailable = computed(
  () =>
    props.availability &&
    (selection.value.skillIds.some((id) => !props.availability?.skills.some((skill) => skill.id === id)) ||
      selection.value.connections.some(
        (selected) =>
          !props.availability?.connections.some((connection) => connection.id === selected.connectionId),
      )),
);
function toggleConnection(connection: McpConnection) {
  const id = connection.id;
  const selected = isConnectionSelected(id);
  selection.value = {
    ...selection.value,
    connections: selected
      ? selection.value.connections.filter((candidate) => candidate.connectionId !== id)
      : [
          ...selection.value.connections,
          {
            connectionId: id,
            toolNames: connection.tools.filter((tool) => tool.policy !== "deny").map((tool) => tool.name),
            resourceUris: [],
            resourceTemplates: [],
            promptNames: [],
          },
        ],
  };
}
function toggleSkill(skillId: string) {
  const previous = selection.value.skillIds;
  selection.value = {
    ...selection.value,
    skillIds: previous.includes(skillId)
      ? previous.filter((value) => value !== skillId)
      : [...previous, skillId],
  };
}
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
        <p v-if="unavailable" role="alert" class="text-caption text-fg-warning">
          {{ $t("spaces.conversation.extensions.unavailable") }}
          <UiButton
            size="sm"
            variant="ghost"
            :disabled="disabled"
            data-testid="clear-conversation-extensions"
            @click="selection = { skillIds: [], connections: [] }"
            >{{ $t("spaces.conversation.extensions.clearSelection") }}</UiButton
          >
        </p>
        <section v-if="availability?.skills.length" class="space-y-1">
          <h3 class="pb-1 text-caption font-medium text-fg-tertiary">
            {{ $t("extensions.overview.skills") }}
          </h3>
          <label
            v-for="skill in availability.skills"
            :key="skill.id"
            class="flex cursor-default items-start gap-2.5 rounded-md p-2 hover:bg-surface-control-hover"
          >
            <UiCheckbox
              class="mt-0.5"
              :model-value="selection.skillIds.includes(skill.id)"
              :label="skill.name"
              @update:model-value="toggleSkill(skill.id)"
            />
            <span class="min-w-0 flex-1">
              <span class="block text-label font-medium break-words text-fg-primary">{{ skill.name }}</span>
              <span
                class="mt-0.5 line-clamp-2 text-caption break-words text-fg-secondary"
                :title="skill.description"
                >{{ skill.description }}</span
              >
            </span>
          </label>
        </section>
        <section v-if="availability?.connections.length" class="space-y-1">
          <h3 class="pb-1 text-caption font-medium text-fg-tertiary">
            {{ $t("extensions.overview.mcp") }}
          </h3>
          <label
            v-for="connection in availability.connections"
            :key="connection.id"
            class="flex cursor-default items-center gap-2.5 rounded-md p-2 text-label text-fg-primary hover:bg-surface-control-hover"
          >
            <UiCheckbox
              :model-value="isConnectionSelected(connection.id)"
              :label="connection.name"
              @update:model-value="toggleConnection(connection)"
            />
            <span class="min-w-0 flex-1 truncate">{{ connection.name }}</span>
          </label>
          <ConversationConnectionSelection
            v-for="connection in selectedConnections"
            :key="connection.id"
            v-model="selection"
            :connection="connection"
          />
        </section>
        <p
          v-if="availability && !availability.skills.length && !availability.connections.length"
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
