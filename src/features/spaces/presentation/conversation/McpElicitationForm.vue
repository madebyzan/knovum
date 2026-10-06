<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed, reactive, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { McpUserInteraction } from "../../contracts/generated/McpUserInteraction";
import { mcpElicitationSchema, type McpElicitationValue } from "../../conversation/mcp-elicitation-form";
import McpElicitationField from "./McpElicitationField.vue";
const props = defineProps<{ interaction: McpUserInteraction; busy: boolean }>();
const emit = defineEmits<{
  answer: [
    id: string,
    response: { action: "accept" | "decline"; content?: Record<string, McpElicitationValue> },
  ];
}>();
const parsed = computed(() => mcpElicitationSchema.safeParse(props.interaction.request));
const request = computed(() => parsed.value.data);
const values = reactive<Record<string, McpElicitationValue>>({});
const error = shallowRef("");
watch(
  request,
  (form) => {
    for (const key of Object.keys(values)) delete values[key];
    if (form?.mode === "form")
      for (const [name, field] of Object.entries(form.requestedSchema.properties)) {
        if (field.default !== undefined) values[name] = field.default;
        else if (field.type === "boolean") values[name] = false;
        else if (field.type === "array") values[name] = [];
      }
  },
  { immediate: true },
);
function accept() {
  if (request.value?.mode === "form") {
    const content: Record<string, McpElicitationValue> = {};
    for (const [name, field] of Object.entries(request.value.requestedSchema.properties)) {
      const value = values[name];
      if (value === undefined || value === "") continue;
      content[name] = field.type === "number" || field.type === "integer" ? Number(value) : value;
    }
    emit("answer", props.interaction.id, { action: "accept", content });
  } else emit("answer", props.interaction.id, { action: "accept" });
}
async function open() {
  if (request.value?.mode !== "url") return;
  try {
    await openUrl(request.value.url);
  } catch (cause) {
    error.value = getErrorMessage(cause);
  }
}
</script>

<template>
  <form
    class="space-y-2 border-t border-stroke-divider p-3"
    @submit.prevent="accept"
    data-testid="mcp-elicitation-form"
  >
    <h3 class="text-body font-medium">{{ $t("extensions.externalUserInput") }}</h3>
    <p v-if="request" class="text-caption">{{ request.message }}</p>
    <p v-else role="alert" class="text-caption text-fg-danger">
      {{ $t("extensions.interactionFieldsInvalid") }}
    </p>
    <template v-if="request?.mode === 'form'">
      <McpElicitationField
        v-for="(field, name) in request.requestedSchema.properties"
        :key="name"
        v-model="values[name]"
        :name="name"
        :field="field"
        :required="request.requestedSchema.required?.includes(name) ?? false"
      />
    </template>
    <template v-if="request?.mode === 'url'"
      ><p class="text-caption break-all text-fg-secondary">{{ request.url }}</p>
      <UiButton size="sm" :disabled="busy" @click="open">{{
        $t("extensions.openExternalPage")
      }}</UiButton></template
    >
    <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
    <div class="flex gap-2">
      <UiButton type="submit" size="sm" :disabled="busy || !request">{{ $t("extensions.accept") }}</UiButton
      ><UiButton
        size="sm"
        variant="ghost"
        :disabled="busy"
        @click="emit('answer', interaction.id, { action: 'decline' })"
        >{{ $t("extensions.decline") }}</UiButton
      >
    </div>
  </form>
</template>
