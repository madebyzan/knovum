<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import type { AgentDescriptor } from "../../../foundation/ai/agent-models";

const props = defineProps<{ agent: AgentDescriptor }>();
const { t } = useI18n();
const connectionMessageKeys: Record<AgentDescriptor["help"]["connection"], string> = {
  native_cli: "settings.ai.agentConnections.nativeCli",
  node_adapter: "settings.ai.agentConnections.nodeAdapter",
  standalone_server: "settings.ai.agentConnections.standaloneServer",
};
const guidance = computed(() => connectionMessageKeys[props.agent.help.connection]);
const links = computed(() => [
  { label: "settings.ai.agentOfficialDocumentation", url: props.agent.help.documentationUrl },
  { label: "settings.ai.agentAcpSetup", url: props.agent.help.setupUrl },
]);

async function visit(url: string) {
  try {
    await openUrl(url);
  } catch (error) {
    toast.error(t("settings.ai.agentDocumentationOpenFailed"), { description: String(error) });
  }
}
</script>

<template>
  <div class="grid gap-2 text-caption text-foreground-secondary">
    <p class="text-label text-foreground">{{ t(`${guidance}.title`) }}</p>
    <p>{{ t(`${guidance}.requirements`) }}</p>
    <p>{{ t("settings.ai.agentEntryNames", { commands: agent.executableNames.join(", ") }) }}</p>
    <p>{{ t("settings.ai.agentDetectionBoundary") }}</p>
    <div class="flex flex-wrap gap-x-3 gap-y-1">
      <a
        v-for="link in links"
        :key="link.url"
        :href="link.url"
        class="rounded-control text-accent underline-offset-2 hover:underline focus-visible:outline-2 focus-visible:outline-accent"
        @click.prevent="visit(link.url)"
        >{{ t(link.label) }}</a
      >
    </div>
  </div>
</template>
