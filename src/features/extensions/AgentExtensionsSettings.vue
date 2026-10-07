<script setup lang="ts">
import { TabsRoot, TabsList, TabsTrigger, TabsContent } from "reka-ui";

import ConnectorSettings from "./connectors/ConnectorSettings.vue";
import PluginSettings from "./plugins/PluginSettings.vue";
import SkillSettings from "./skills/SkillSettings.vue";

const tabs = ["plugins", "skills", "connectors"] as const;
</script>

<template>
  <section data-testid="agent-extensions-settings" class="min-w-0 space-y-5">
    <p class="max-w-xl text-body text-fg-secondary">{{ $t("extensions.description") }}</p>
    <TabsRoot default-value="plugins">
      <TabsList
        class="mb-5 inline-flex max-w-full gap-0.5 rounded-md bg-surface-control p-1"
        :aria-label="$t('extensions.title')"
      >
        <TabsTrigger
          v-for="tab in tabs"
          :key="tab"
          :value="tab"
          :data-testid="`extensions-tab-${tab}`"
          class="inline-flex h-(--knovum-control-height-sm) items-center justify-center rounded-sm px-3 text-label font-medium text-fg-secondary hover:text-fg-primary data-[state=active]:bg-surface-raised data-[state=active]:text-fg-primary data-[state=active]:shadow-control"
        >
          {{ $t(`extensions.${tab}`) }}
        </TabsTrigger>
      </TabsList>
      <TabsContent
        value="plugins"
        class="outline-none focus-visible:outline-2 focus-visible:outline-stroke-focus"
        ><PluginSettings
      /></TabsContent>
      <TabsContent
        value="skills"
        class="outline-none focus-visible:outline-2 focus-visible:outline-stroke-focus"
        ><SkillSettings
      /></TabsContent>
      <TabsContent
        value="connectors"
        class="outline-none focus-visible:outline-2 focus-visible:outline-stroke-focus"
        ><ConnectorSettings
      /></TabsContent>
    </TabsRoot>
  </section>
</template>
