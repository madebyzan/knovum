<script setup lang="ts">
import { TabsRoot, TabsList, TabsTrigger, TabsContent } from "reka-ui";
import { computed } from "vue";

import type { McpEndpoint } from "../../../shared/contracts/generated/McpEndpoint";
import McpHttpFields from "./McpHttpFields.vue";
import McpProcessFields from "./McpProcessFields.vue";
const endpoint = defineModel<McpEndpoint>({ required: true });
defineProps<{ disabled?: boolean }>();
const emit = defineEmits<{ changeTransport: [] }>();
const httpEndpoint = computed(() => (endpoint.value.type === "streamable_http" ? endpoint.value : undefined));
const processEndpoint = computed(() => (endpoint.value.type === "stdio" ? endpoint.value : undefined));
function setType(type: string | number) {
  if (type === endpoint.value.type) return;
  if (type !== "stdio" && type !== "streamable_http") return;
  endpoint.value =
    type === "stdio"
      ? { type: "stdio", executable: "", args: [], cwd: null, environment: {} }
      : { type: "streamable_http", url: "", headers: {}, allowLocalNetwork: false };
  emit("changeTransport");
}
</script>
<template>
  <div class="space-y-4">
    <TabsRoot :model-value="endpoint.type" class="space-y-4" @update:model-value="setType">
      <div class="flex items-center justify-between gap-3">
        <span class="text-label font-medium">{{ $t("extensions.connections.transport") }}</span>
        <TabsList
          :aria-label="$t('extensions.connections.transport')"
          class="inline-flex gap-1 rounded-lg bg-surface-control p-1"
        >
          <TabsTrigger
            v-for="transport in ['stdio', 'streamable_http']"
            :key="transport"
            :value="transport"
            :disabled="disabled"
            :data-testid="`mcp-transport-${transport}`"
            class="rounded-md px-3 py-1 text-label text-fg-secondary outline-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-40 data-[state=active]:bg-surface-overlay data-[state=active]:text-fg-primary"
          >
            {{ transport === "stdio" ? "STDIO" : "Streamable HTTP" }}
          </TabsTrigger>
        </TabsList>
      </div>
      <TabsContent value="streamable_http" class="outline-none">
        <McpHttpFields
          v-if="httpEndpoint"
          :model-value="httpEndpoint"
          @update:model-value="endpoint = $event"
          :disabled="disabled"
        />
      </TabsContent>
      <TabsContent value="stdio" class="outline-none">
        <McpProcessFields
          v-if="processEndpoint"
          :model-value="processEndpoint"
          @update:model-value="endpoint = $event"
          :disabled="disabled"
        />
      </TabsContent>
    </TabsRoot>
  </div>
</template>
