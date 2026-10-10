<script setup lang="ts">
import { Server } from "@lucide/vue";
import { useId } from "vue";

import type { ConnectorDefinition } from "../../../../shared/contracts/generated/ConnectorDefinition";
import type { McpEndpoint } from "../../../../shared/contracts/generated/McpEndpoint";
import UiButton from "../../../../shared/ui/UiButton.vue";
import ExtensionInspectionHeader from "../../components/ExtensionInspectionHeader.vue";
import type { EndpointField } from "../../connections/endpoint-fields";
import ConnectorEditor from "./ConnectorEditor.vue";
const draft = defineModel<ConnectorDefinition & { endpoint: McpEndpoint }>({ required: true });
const endpointFields = defineModel<EndpointField[]>("endpointFields", { required: true });
const formId = useId();
defineProps<{ busy: boolean; title: string; error?: string | undefined }>();
defineEmits<{ save: []; close: [] }>();
</script>
<template>
  <section class="max-w-xl space-y-5" data-testid="mcp-server-configuration">
    <ExtensionInspectionHeader :name="title" :icon="Server" @back="$emit('close')" />
    <ConnectorEditor
      v-model="draft"
      v-model:endpoint-fields="endpointFields"
      :busy="busy"
      :form-id="formId"
      @save="$emit('save')"
    />
    <p v-if="error" role="alert" class="mt-3 text-body text-fg-danger">{{ error }}</p>
    <footer class="flex justify-end gap-2 border-t border-stroke-divider pt-4">
      <UiButton :disabled="busy" variant="ghost" @click="$emit('close')">{{ $t("common.cancel") }}</UiButton>
      <UiButton
        variant="primary"
        type="submit"
        :form="formId"
        :disabled="busy || !draft.name.trim()"
        data-testid="save-connector"
        >{{ $t("common.save") }}</UiButton
      >
    </footer>
  </section>
</template>
