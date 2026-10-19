<script setup lang="ts">
import { AlertTriangle, Wrench } from "@lucide/vue";

import type { McpConnectionCatalog } from "../../../../shared/contracts/generated/McpConnectionCatalog";
import type { McpToolPermission } from "../../../../shared/contracts/generated/McpToolPermission";
import type { McpToolPolicy } from "../../../../shared/contracts/generated/McpToolPolicy";
import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiExpandableText from "../../../../shared/ui/UiExpandableText.vue";
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
  <section class="space-y-4" data-testid="mcp-tool-selection">
    <div
      class="flex items-start gap-2.5 rounded-lg border border-stroke-warning/30 bg-surface-warning-subtle p-3 text-caption text-fg-warning"
    >
      <AlertTriangle class="mt-0.5 size-4 shrink-0" />
      <span class="leading-relaxed">{{ $t("extensions.connections.tools.policyHint") }}</span>
    </div>

    <div class="space-y-3">
      <div
        v-for="tool in catalog.tools"
        :key="tool.name"
        class="rounded-panel border border-stroke-default bg-surface-raised p-4 transition-colors hover:border-stroke-default-hover"
      >
        <div class="flex flex-wrap items-center justify-between gap-2">
          <div class="flex items-center gap-2">
            <span
              class="flex size-6 shrink-0 items-center justify-center rounded bg-surface-control text-fg-secondary"
            >
              <Wrench class="size-3.5" />
            </span>
            <span class="font-mono text-body font-semibold text-fg-primary">{{ tool.name }}</span>
          </div>
          <div class="flex flex-wrap items-center gap-1.5">
            <UiBadge v-if="tool.readOnlyHint" size="sm" variant="neutral">
              {{ $t("extensions.connections.tools.serverReadOnlyHint") }}
            </UiBadge>
            <UiBadge
              size="sm"
              :variant="
                permission(tool.name)?.policy === 'allow'
                  ? 'success'
                  : permission(tool.name)?.policy === 'confirm'
                    ? 'accent'
                    : 'neutral'
              "
            >
              {{ $t(`extensions.connections.tools.${permission(tool.name)?.policy ?? 'deny'}`) }}
            </UiBadge>
          </div>
        </div>

        <UiExpandableText
          v-if="tool.description"
          :text="tool.description"
          :clamp-lines="2"
          class="mt-2 text-caption leading-relaxed text-fg-secondary"
        />

        <div class="mt-3 flex flex-wrap items-center justify-between gap-3 border-t border-stroke-divider/80 pt-3">
          <div class="flex items-center gap-2">
            <span class="text-caption text-fg-secondary">{{ $t("extensions.connections.tools.policy") }}</span>
            <UiSelect
              :model-value="permission(tool.name)?.policy ?? 'deny'"
              :label="$t('extensions.connections.tools.policy')"
              class="w-32"
              size="sm"
              :disabled="busy"
              :options="[
                { value: 'deny', label: $t('extensions.connections.tools.deny') },
                { value: 'confirm', label: $t('extensions.connections.tools.confirm') },
                { value: 'allow', label: $t('extensions.connections.tools.allow') },
              ]"
              @update:model-value="change(tool.name, $event, permission(tool.name)?.readOnly ?? false)"
            />
          </div>
          <label
            class="flex cursor-pointer select-none items-center gap-2 text-caption text-fg-secondary transition-colors hover:text-fg-primary"
          >
            <UiCheckbox
              :model-value="permission(tool.name)?.readOnly ?? false"
              :label="$t('extensions.connections.tools.reviewedReadOnly')"
              :disabled="busy"
              @update:model-value="change(tool.name, permission(tool.name)?.policy ?? 'deny', $event)"
            />
            <span>{{ $t("extensions.connections.tools.reviewedReadOnly") }}</span>
          </label>
        </div>
      </div>
    </div>

    <div class="flex flex-wrap items-center justify-between gap-3 pt-1">
      <span class="text-caption text-fg-tertiary">
        {{
          $t("extensions.catalog.counts", {
            resources: catalog.resources.length,
            prompts: catalog.prompts.length,
            templates: catalog.resourceTemplates.length,
          })
        }}
      </span>
      <UiButton
        variant="primary"
        size="sm"
        :disabled="busy || !canSave"
        data-testid="save-mcp-tool-permissions"
        @click="emit('save')"
      >
        {{ $t("extensions.connections.tools.savePermissions") }}
      </UiButton>
    </div>
  </section>
</template>
