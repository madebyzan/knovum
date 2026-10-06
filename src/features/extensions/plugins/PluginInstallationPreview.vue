<script setup lang="ts">
import type { PluginInspection } from "../../../shared/contracts/generated/PluginInspection";
import UiButton from "../../../shared/ui/UiButton.vue";
defineProps<{ inspection: PluginInspection; busy: boolean; updating: boolean }>();
const emit = defineEmits<{ install: []; cancel: [] }>();
</script>

<template>
  <section
    data-testid="plugin-installation-preview"
    class="space-y-3 rounded-sm border border-stroke-divider p-3"
  >
    <h2 class="text-body font-medium">{{ inspection.manifest.name }} {{ inspection.manifest.version }}</h2>
    <p class="text-caption text-fg-secondary">{{ inspection.manifest.description }}</p>
    <p class="text-caption break-all text-fg-secondary">SHA-256: {{ inspection.digest }}</p>
    <p class="text-caption text-fg-warning">{{ $t("extensions.installDoesNotAuthorize") }}</p>
    <ul class="space-y-1 text-body">
      <li v-for="component in inspection.components" :key="`${component.kind}:${component.name}`">
        {{ component.kind }} · {{ component.name
        }}<span v-if="component.diagnostic" class="ml-2 text-fg-warning">{{ component.diagnostic }}</span>
      </li>
    </ul>
    <section
      v-for="connection in inspection.connections"
      :key="connection.name"
      class="space-y-1 text-caption"
    >
      <h3 class="font-medium">{{ connection.name }}</h3>
      <pre class="max-h-40 overflow-auto whitespace-pre-wrap">{{
        JSON.stringify(
          { configuration: connection.configuration, authorization: connection.authorization },
          null,
          2,
        )
      }}</pre>
    </section>
    <div class="flex gap-2">
      <UiButton :disabled="busy" data-testid="confirm-plugin-installation" @click="emit('install')">{{
        $t(updating ? "extensions.update" : "extensions.install")
      }}</UiButton>
      <UiButton variant="ghost" @click="emit('cancel')">{{ $t("extensions.cancel") }}</UiButton>
    </div>
  </section>
</template>
