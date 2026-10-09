<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { readMcpResource } from "../../../../foundation/extensions/connection-operations";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import type { McpResourceSelection } from "../../../../shared/contracts/generated/McpResourceSelection";
import UiButton from "../../../../shared/ui/UiButton.vue";
import { importMcpResource } from "../../assets/mcp-resource-import";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { useSpaceStore } from "../../session/space-store";
const props = defineProps<{
  connection: McpConnection;
  catalogDigest: string;
  selection: McpResourceSelection;
}>();
const store = useSpaceStore();
const gate = useNodeOperationGate();
const { t } = useI18n();
const reading = useMutation({
  mutationFn: () => readMcpResource(props.connection, props.catalogDigest, props.selection),
});
const importing = useMutation({
  mutationFn: (index: number) => {
    if (!gate.allows("create")) throw new Error(t("spaces.conversation.extensionResources.readOnly"));
    return importMcpResource({
      store,
      connection: props.connection,
      selection: props.selection,
      content: reading.data.value!,
      contentIndex: index,
      labels: {
        title: t("spaces.importExport.progress.resource"),
        preparing: t("spaces.importExport.progress.preparing"),
      },
    });
  },
});
</script>

<template>
  <section class="space-y-2 rounded-sm border border-stroke-divider p-2" data-testid="mcp-resource-preview">
    <UiButton
      size="sm"
      variant="ghost"
      :disabled="reading.isPending.value || importing.isPending.value"
      @click="reading.mutate()"
      >{{ $t("extensions.catalog.read") }}</UiButton
    >
    <p v-if="reading.error.value || importing.error.value" role="alert" class="text-fg-danger">
      {{ getErrorMessage(reading.error.value || importing.error.value) }}
    </p>
    <template v-if="reading.data.value">
      <p class="break-all text-fg-secondary">{{ reading.data.value.uri }}</p>
      <section v-for="(content, index) in reading.data.value.contents" :key="index" class="space-y-1">
        <pre class="max-h-48 overflow-auto break-all whitespace-pre-wrap">{{
          "text" in content
            ? content.text
            : JSON.stringify(
                { uri: content.uri, mimeType: content.mimeType, binary: "blob" in content },
                null,
                2,
              )
        }}</pre>
        <UiButton
          size="sm"
          :disabled="importing.isPending.value || !gate.allows('create')"
          @click="importing.mutate(index)"
          >{{ $t("spaces.conversation.extensionResources.import") }}</UiButton
        >
      </section>
    </template>
  </section>
</template>
