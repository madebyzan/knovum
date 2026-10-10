<script setup lang="ts">
import type { McpConnectionCatalog } from "../../../../shared/contracts/generated/McpConnectionCatalog";
import type { McpToolPermission } from "../../../../shared/contracts/generated/McpToolPermission";
import type { McpToolPolicy } from "../../../../shared/contracts/generated/McpToolPolicy";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
const permissions = defineModel<McpToolPermission[]>({ required: true });
const props = defineProps<{ catalog: McpConnectionCatalog; busy: boolean; canSave: boolean }>();
const emit = defineEmits<{ save: [] }>();
function permission(name: string) {
  return permissions.value.find((candidate) => candidate.name === name);
}
function change(name: string, policy: McpToolPolicy, readOnly: boolean) {
  const tool = props.catalog.tools.find((candidate) => candidate.name === name)!;
  permissions.value = [
    ...permissions.value.filter((candidate) => candidate.name !== name),
    { name, policy, readOnly, schemaDigest: tool.schemaDigest },
  ];
}
</script>

<template>
  <section class="space-y-3" data-testid="mcp-tool-selection">
    <p class="text-caption text-fg-warning">{{ $t("extensions.connections.tools.policyHint") }}</p>
    <div v-for="tool in catalog.tools" :key="tool.name" class="space-y-2 border-b border-stroke-divider py-3">
      <h3 class="text-body font-medium">{{ tool.name }}</h3>
      <p class="text-caption text-fg-secondary">{{ tool.description }}</p>
      <div class="flex flex-wrap gap-3 text-caption">
        <UiSelect
          :model-value="permission(tool.name)?.policy ?? 'deny'"
          :label="$t('extensions.connections.tools.policy')"
          class="w-40"
          size="sm"
          :disabled="busy"
          :options="[
            { value: 'deny', label: $t('extensions.connections.tools.deny') },
            { value: 'confirm', label: $t('extensions.connections.tools.confirm') },
            { value: 'allow', label: $t('extensions.connections.tools.allow') },
          ]"
          @update:model-value="change(tool.name, $event, permission(tool.name)?.readOnly ?? false)"
        />
        <label class="flex items-center gap-2"
          ><UiCheckbox
            :model-value="permission(tool.name)?.readOnly ?? false"
            :label="$t('extensions.connections.tools.reviewedReadOnly')"
            :disabled="busy"
            @update:model-value="change(tool.name, permission(tool.name)?.policy ?? 'deny', $event)"
          />{{ $t("extensions.connections.tools.reviewedReadOnly") }}</label
        >
        <span v-if="tool.readOnlyHint" class="text-fg-tertiary">{{
          $t("extensions.connections.tools.serverReadOnlyHint")
        }}</span>
      </div>
    </div>
    <p class="text-caption text-fg-secondary">
      {{
        $t("extensions.catalog.counts", {
          resources: catalog.resources.length,
          prompts: catalog.prompts.length,
          templates: catalog.resourceTemplates.length,
        })
      }}
    </p>
    <UiButton :disabled="busy || !canSave" data-testid="save-mcp-tool-permissions" @click="emit('save')">{{
      $t("extensions.connections.tools.savePermissions")
    }}</UiButton>
  </section>
</template>
