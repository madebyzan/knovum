<script setup lang="ts">
import { ref } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { resourceParameters } from "../../../../foundation/extensions/catalog-input";
import type { McpResourceTemplateSelection } from "../../../../shared/contracts/generated/McpResourceTemplateSelection";
import UiButton from "../../../../shared/ui/UiButton.vue";
const props = defineProps<{ uriTemplate: string }>();
const emit = defineEmits<{ select: [selection: McpResourceTemplateSelection] }>();
const parameters = ref("{}");
const error = ref("");
function select() {
  try {
    const values = resourceParameters(parameters.value);
    error.value = "";
    emit("select", { uriTemplate: props.uriTemplate, parameters: values });
  } catch (failure) {
    error.value = getErrorMessage(failure);
  }
}
</script>

<template>
  <form class="space-y-1" @submit.prevent="select">
    <label class="block"
      >{{ uriTemplate
      }}<textarea
        v-model="parameters"
        :aria-label="$t('extensions.parameters')"
        class="block w-full rounded-sm bg-surface-control p-1 font-mono"
        rows="2"
      />
    </label>
    <p v-if="error" role="alert" class="text-fg-danger">{{ error }}</p>
    <UiButton type="submit" size="sm" variant="ghost">{{ $t("extensions.selectTemplateResource") }}</UiButton>
  </form>
</template>
