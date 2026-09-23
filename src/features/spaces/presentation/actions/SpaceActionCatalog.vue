<script setup lang="ts">
import { computed, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { listSpaceActions, type SpaceActionDefinition } from "../../../../foundation/ai/space-action-catalog";
import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";

const { t } = useI18n();
const query = shallowRef("");
const actions = shallowRef<SpaceActionDefinition[]>([]);
const selectedId = shallowRef("");
const filtered = computed(() => {
  const normalized = query.value.trim().toLocaleLowerCase();
  return normalized
    ? actions.value.filter((action) => action.id.toLocaleLowerCase().includes(normalized))
    : actions.value;
});
const selected = computed(() => actions.value.find(({ id }) => id === selectedId.value) ?? filtered.value[0]);
const parameterRows = computed(() => {
  const properties = selected.value?.parametersSchema.properties;
  if (!properties || typeof properties !== "object" || Array.isArray(properties)) return [];
  const required = new Set(
    Array.isArray(selected.value?.parametersSchema.required)
      ? selected.value.parametersSchema.required.filter((item): item is string => typeof item === "string")
      : [],
  );
  return Object.entries(properties).map(([key, schema]) => ({
    key,
    required: required.has(key),
    type:
      schema && typeof schema === "object" && !Array.isArray(schema) && "type" in schema
        ? String(schema.type)
        : "value",
  }));
});

onMounted(async () => {
  actions.value = await listSpaceActions();
  selectedId.value = actions.value[0]?.id ?? "";
});
</script>

<template>
  <section class="grid h-full min-h-0 grid-cols-[minmax(12rem,0.8fr)_minmax(16rem,1.2fr)]">
    <aside class="min-h-0 overflow-y-auto border-r border-divider p-2">
      <UiInput v-model="query" size="sm" :placeholder="t('spaces.assistant.searchActions')" />
      <button
        v-for="action in filtered"
        :key="action.id"
        type="button"
        class="mt-1 flex w-full items-center justify-between rounded-control px-2 py-1.5 text-left text-label hover:bg-control"
        :class="selected?.id === action.id ? 'bg-control text-body' : 'text-foreground-secondary'"
        @click="selectedId = action.id"
      >
        <span class="truncate">{{ action.id }}</span>
        <span class="text-micro">v{{ action.version }}</span>
      </button>
      <p v-if="!filtered.length" class="p-3 text-caption text-foreground-secondary">
        {{ t("spaces.assistant.actionEmpty") }}
      </p>
    </aside>
    <article v-if="selected" class="min-h-0 overflow-y-auto p-4">
      <div class="flex flex-wrap items-center gap-1.5">
        <h2 class="mr-auto text-body text-title-sm">{{ selected.id }}</h2>
        <UiBadge size="sm">v{{ selected.version }}</UiBadge>
        <UiBadge size="sm">{{ selected.impact }}</UiBadge>
        <UiBadge size="sm">{{ selected.executionMode }}</UiBadge>
      </div>
      <dl class="mt-4 grid grid-cols-[8rem_1fr] gap-x-3 gap-y-2 text-caption">
        <dt class="text-foreground-secondary">{{ t("spaces.assistant.approvalPolicy") }}</dt>
        <dd>{{ selected.approvalPolicy }}</dd>
        <dt class="text-foreground-secondary">{{ t("spaces.assistant.inputConsistency") }}</dt>
        <dd>{{ selected.inputConsistency }}</dd>
        <dt class="text-foreground-secondary">{{ t("spaces.assistant.executionCapability") }}</dt>
        <dd>{{ selected.executionCapability ?? "—" }}</dd>
        <dt class="text-foreground-secondary">{{ t("spaces.assistant.inputRoles") }}</dt>
        <dd>{{ selected.inputRoles.join(", ") || "—" }}</dd>
        <dt class="text-foreground-secondary">{{ t("spaces.assistant.outputContract") }}</dt>
        <dd class="font-mono break-all">{{ selected.outputContract }}</dd>
      </dl>
      <h3 class="mt-5 text-body text-label font-medium">{{ t("spaces.assistant.parameters") }}</h3>
      <div class="mt-2 divide-y divide-divider rounded-panel border border-border">
        <div
          v-for="parameter in parameterRows"
          :key="parameter.key"
          class="flex justify-between gap-3 px-3 py-2 text-caption"
        >
          <code>{{ parameter.key }}{{ parameter.required ? " *" : "" }}</code
          ><span class="text-foreground-secondary">{{ parameter.type }}</span>
        </div>
        <p v-if="!parameterRows.length" class="px-3 py-2 text-caption text-foreground-secondary">—</p>
      </div>
      <p class="mt-4 text-caption text-foreground-secondary">{{ t("spaces.assistant.actionCatalogHint") }}</p>
    </article>
  </section>
</template>
