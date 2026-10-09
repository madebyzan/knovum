<script setup lang="ts">
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpEndpoint } from "../../../shared/contracts/generated/McpEndpoint";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import ConnectorEditor from "./ConnectorEditor.vue";
const draft = defineModel<ConnectorDefinition & { endpoint: McpEndpoint }>({ required: true });
const publicFields = defineModel<string>("publicFields", { required: true });
defineProps<{ open: boolean; busy: boolean; title: string; error?: string | undefined }>();
defineEmits<{ save: []; close: [] }>();
</script>
<template>
  <UiDialog
    :open="open"
    :title="title"
    size="wide"
    :closable="!busy"
    @update:open="!$event && $emit('close')"
  >
    <ConnectorEditor
      v-if="open"
      v-model="draft"
      v-model:public-fields="publicFields"
      :busy="busy"
      @save="$emit('save')"
      @close="$emit('close')"
    />
    <p v-if="error" role="alert" class="mt-3 text-body text-fg-danger">{{ error }}</p>
  </UiDialog>
</template>
