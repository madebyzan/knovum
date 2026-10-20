<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { ExternalAgentDescriptor } from "../../../../shared/contracts/generated/ExternalAgentDescriptor";

const props = defineProps<{ agent: ExternalAgentDescriptor }>();
const { t } = useI18n();
const MESSAGE_KEY_BY_CONNECTION_HELP: Record<ExternalAgentDescriptor["help"]["connection"], string> = {
  executable_cli: "settings.agents.connections.executableCli",
  node_adapter: "settings.agents.connections.nodeAdapter",
  standalone_server: "settings.agents.connections.standaloneServer",
};
const guidance = computed(() => MESSAGE_KEY_BY_CONNECTION_HELP[props.agent.help.connection]);
const links = computed(() => [
  { label: "settings.agents.officialDocumentation", url: props.agent.help.documentationUrl },
  { label: "settings.agents.acpSetup", url: props.agent.help.setupUrl },
]);

async function visit(url: string) {
  try {
    await openUrl(url);
  } catch (error) {
    toast.error(t("settings.agents.documentationOpenFailed"), { description: getErrorMessage(error) });
  }
}
</script>

<template>
  <div class="grid gap-2 text-caption text-fg-secondary">
    <p class="text-label text-fg-primary">{{ t(`${guidance}.title`) }}</p>
    <p>{{ t(`${guidance}.requirements`) }}</p>
    <p>{{ t("settings.agents.requiredCommands", { commands: agent.executableNames.join(", ") }) }}</p>
    <p>{{ t("settings.agents.detectionBoundary") }}</p>
    <div class="flex flex-wrap gap-x-3 gap-y-1">
      <a
        v-for="link in links"
        :key="link.url"
        :href="link.url"
        class="rounded-md text-fg-accent underline-offset-2 hover:underline focus-visible:outline-2 focus-visible:outline-stroke-accent"
        @click.prevent="visit(link.url)"
        >{{ t(link.label) }}</a
      >
    </div>
  </div>
</template>
