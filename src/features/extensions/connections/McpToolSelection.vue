<script setup lang="ts">
import type { McpConnectionCatalog } from "../../../shared/contracts/generated/McpConnectionCatalog";
import type { McpToolPermission } from "../../../shared/contracts/generated/McpToolPermission";
import type { McpToolPolicy } from "../../../shared/contracts/generated/McpToolPolicy";
import UiButton from "../../../shared/ui/UiButton.vue";
const permissions = defineModel<McpToolPermission[]>({ required: true });
const props = defineProps<{ catalog: McpConnectionCatalog; busy: boolean }>();
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
function changePolicy(name: string, event: Event) {
  change(
    name,
    (event.target as HTMLSelectElement).value as McpToolPolicy,
    permission(name)?.readOnly ?? false,
  );
}
function changeReadOnly(name: string, event: Event) {
  change(name, permission(name)?.policy ?? "deny", (event.target as HTMLInputElement).checked);
}
</script>

<template>
  <section class="space-y-3 rounded-sm border border-stroke-divider p-3" data-testid="mcp-tool-selection">
    <p class="text-caption text-fg-warning">{{ $t("extensions.toolPermissionPolicy") }}</p>
    <div v-for="tool in catalog.tools" :key="tool.name" class="space-y-1 border-b border-stroke-divider py-2">
      <h3 class="text-body font-medium">{{ tool.name }}</h3>
      <p class="text-caption text-fg-secondary">{{ tool.description }}</p>
      <div class="flex flex-wrap gap-3 text-caption">
        <label
          >{{ $t("extensions.policy")
          }}<select
            :value="permission(tool.name)?.policy ?? 'deny'"
            class="ml-2 rounded-sm bg-surface-control p-1"
            @change="changePolicy(tool.name, $event)"
          >
            <option value="deny">{{ $t("extensions.deny") }}</option>
            <option value="confirm">{{ $t("extensions.confirm") }}</option>
            <option value="allow">{{ $t("extensions.allow") }}</option>
          </select></label
        >
        <label class="flex items-center gap-1"
          ><input
            type="checkbox"
            :checked="permission(tool.name)?.readOnly ?? false"
            @change="changeReadOnly(tool.name, $event)"
          />{{ $t("extensions.confirmReadOnly") }}</label
        >
        <span v-if="tool.readOnlyHint" class="text-fg-tertiary">{{
          $t("extensions.serverReadOnlyHint")
        }}</span>
      </div>
    </div>
    <p class="text-caption text-fg-secondary">
      {{
        $t("extensions.catalogCounts", {
          resources: catalog.resources.length,
          prompts: catalog.prompts.length,
          templates: catalog.resourceTemplates.length,
        })
      }}
    </p>
    <UiButton :disabled="busy" data-testid="save-mcp-tool-permissions" @click="emit('save')">{{
      $t("extensions.savePermissions")
    }}</UiButton>
  </section>
</template>
