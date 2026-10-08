<script setup lang="ts">
import { Blocks, ShieldCheck } from "@lucide/vue";

import type { PluginInspection } from "../../../shared/contracts/generated/PluginInspection";
import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
defineProps<{ inspection: PluginInspection; busy: boolean; updating: boolean }>();
const emit = defineEmits<{ install: []; cancel: [] }>();
</script>

<template>
  <section data-testid="plugin-installation-preview" class="space-y-5">
    <div class="flex items-start gap-3">
      <div
        class="flex size-10 shrink-0 items-center justify-center rounded-lg bg-surface-muted text-fg-secondary"
      >
        <Blocks class="size-5" :stroke-width="1.5" />
      </div>
      <div class="min-w-0">
        <h3 class="text-title-sm font-semibold break-words">
          {{ inspection.manifest.name }}
          <span class="text-body font-normal text-fg-tertiary">{{ inspection.manifest.version }}</span>
        </h3>
        <p class="mt-1 text-body text-fg-secondary">{{ inspection.manifest.description }}</p>
      </div>
    </div>
    <p class="flex items-start gap-2 rounded-lg bg-surface-muted p-3 text-caption text-fg-secondary">
      <ShieldCheck class="mt-0.5 size-4 shrink-0" :stroke-width="1.5" />{{
        $t("extensions.plugins.installation.authorizationHint")
      }}
    </p>
    <section class="space-y-2">
      <h3 class="text-body font-medium">{{ $t("extensions.plugins.capabilities") }}</h3>
      <ul class="divide-y divide-stroke-divider rounded-lg bg-surface-muted px-3">
        <li
          v-for="component in inspection.components"
          :key="`${component.kind}:${component.name}`"
          class="flex flex-wrap items-center gap-2 py-2.5 text-body"
        >
          <UiBadge size="sm">{{ component.kind }}</UiBadge
          ><span class="min-w-0 break-words">{{ component.name }}</span
          ><span v-if="component.diagnostic" class="text-caption text-fg-warning">{{
            component.diagnostic
          }}</span>
        </li>
      </ul>
    </section>
    <details class="text-caption text-fg-secondary">
      <summary
        class="w-fit rounded-sm py-1 outline-none focus-visible:outline-2 focus-visible:outline-stroke-focus"
      >
        {{ $t("extensions.plugins.verification.title") }}
      </summary>
      <p class="mt-2 rounded-md bg-surface-muted p-3 font-mono break-all">SHA-256: {{ inspection.digest }}</p>
    </details>
    <section
      v-for="connector in inspection.connectors"
      :key="connector.componentName"
      class="space-y-2 rounded-lg bg-surface-muted p-3 text-caption"
    >
      <h3 class="font-medium">{{ connector.name }}</h3>
      <p v-if="connector.description" class="text-fg-secondary">{{ connector.description }}</p>
      <pre class="max-h-40 overflow-auto whitespace-pre-wrap">{{
        JSON.stringify(
          { configuration: connector.configuration, authorization: connector.authorization },
          null,
          2,
        )
      }}</pre>
    </section>
    <div class="flex justify-end gap-2 border-t border-stroke-divider pt-4">
      <UiButton
        variant="primary"
        :disabled="busy"
        data-testid="confirm-plugin-installation"
        @click="emit('install')"
        ><UiSpinner v-if="busy" class="size-3.5" />{{
          $t(updating ? "extensions.plugins.update.action" : "extensions.plugins.installation.install")
        }}</UiButton
      >
      <UiButton :disabled="busy" variant="ghost" @click="emit('cancel')">{{ $t("common.cancel") }}</UiButton>
    </div>
  </section>
</template>
