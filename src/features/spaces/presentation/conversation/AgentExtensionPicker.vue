<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";

import type { AgentExtensionAvailability } from "../../../../shared/contracts/generated/AgentExtensionAvailability";
import type { AgentExtensionSelection } from "../../../../shared/contracts/generated/AgentExtensionSelection";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
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
  <UiPopover>
    <template #trigger
      ><UiButton size="sm" variant="ghost" :disabled="disabled" data-testid="conversation-extension-picker"
        >{{ $t("spaces.conversation.extensions.label") }} · {{ count }}</UiButton
      ></template
    >
    <section
      class="max-h-80 w-72 space-y-3 overflow-auto p-3"
      :aria-label="$t('spaces.conversation.extensions.label')"
    >
      <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
      <p v-if="unavailable" role="alert" class="text-caption text-fg-warning">
        {{ $t("spaces.conversation.extensions.unavailable") }}
      </p>
      <h3 class="text-caption font-medium">Skills</h3>
      <label v-for="skill in availability?.skills" :key="skill.id" class="flex items-start gap-2 text-body"
        ><input
          type="checkbox"
          :checked="selection.skillIds.includes(skill.id)"
          @change="toggleSkill(skill.id)"
        /><span
          >{{ skill.name
          }}<span class="block text-caption text-fg-secondary">{{ skill.description }}</span></span
        ></label
      >
      <h3 class="text-caption font-medium">{{ $t("extensions.overview.connectors") }}</h3>
      <label
        v-for="connection in availability?.connections"
        :key="connection.id"
        class="flex items-center gap-2 text-body"
        ><input
          type="checkbox"
          :checked="isConnectionSelected(connection.id)"
          @change="toggleConnection(connection)"
        />{{ connection.name }}</label
      >
      <ConversationConnectionSelection
        v-for="connection in selectedConnections"
        :key="connection.id"
        v-model="selection"
        :connection="connection"
      />
      <p
        v-for="diagnostic in availability?.diagnostics"
        :key="diagnostic"
        class="text-caption text-fg-warning"
      >
        {{ diagnostic }}
      </p>
      <UiButton
        size="sm"
        variant="ghost"
        @click="router.push({ path: '/settings', query: { pane: 'extensions' } })"
        >{{ $t("spaces.conversation.extensions.manage") }}</UiButton
      >
    </section>
  </UiPopover>
</template>
