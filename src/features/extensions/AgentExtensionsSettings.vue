<script setup lang="ts">
import { TabsRoot, TabsList, TabsTrigger, TabsContent } from "reka-ui";

import ConnectorSettings from "./connectors/ConnectorSettings.vue";
import PluginSettings from "./plugins/PluginSettings.vue";
import SkillSettings from "./skills/SkillSettings.vue";
</script>

<template>
  <section data-testid="agent-extensions-settings" class="space-y-4">
    <p class="text-body text-fg-secondary">{{ $t("extensions.description") }}</p>
    <TabsRoot default-value="plugins">
      <TabsList class="mb-4 flex gap-2" :aria-label="$t('extensions.title')">
        <TabsTrigger
          v-for="tab in ['plugins', 'skills', 'connectors']"
          :key="tab"
          :value="tab"
          class="rounded-sm px-3 py-2 text-body text-fg-secondary hover:bg-surface-control data-[state=active]:bg-surface-control data-[state=active]:text-fg-primary"
        >
          {{ $t(`extensions.${tab}`) }}
        </TabsTrigger>
      </TabsList>
      <TabsContent value="plugins"><PluginSettings /></TabsContent>
      <TabsContent value="skills"><SkillSettings /></TabsContent>
      <TabsContent value="connectors"><ConnectorSettings /></TabsContent>
    </TabsRoot>
  </section>
</template>
