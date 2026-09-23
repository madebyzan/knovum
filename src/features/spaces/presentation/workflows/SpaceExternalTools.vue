<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted, reactive, shallowRef, watch } from "vue";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { useSpaceStore } from "../../session/space-store";
import {
  deleteExternalToolBinding,
  disableExternalToolBinding,
  listExternalToolBindings,
  saveExternalToolBinding,
  testExternalToolConnection,
  type ExternalToolBinding,
} from "../../workflows/external-tool-client";

const { activeSpaceId } = storeToRefs(useSpaceStore());
const bindings = shallowRef<ExternalToolBinding[]>([]);
const busy = shallowRef(false);
const error = shallowRef<string>();
const diagnostic = shallowRef<unknown>();
const form = reactive({
  id: "",
  name: "",
  endpoint: "",
  secretKey: "",
  toolName: "",
  effect: "read_only" as ExternalToolBinding["effect"],
});
const effectOptions = computed(() =>
  ["read_only", "additive", "destructive"].map((value) => ({ value, label: value })),
);

async function refresh() {
  bindings.value = activeSpaceId.value ? await listExternalToolBindings(activeSpaceId.value) : [];
}

async function save() {
  if (!activeSpaceId.value) return;
  busy.value = true;
  error.value = undefined;
  try {
    await saveExternalToolBinding(bindingRequest(true));
    Object.assign(form, { id: "", name: "", endpoint: "", secretKey: "", toolName: "" });
    await refresh();
  } catch (reason) {
    error.value = String(reason);
  } finally {
    busy.value = false;
  }
}

function bindingRequest(enabled: boolean) {
  if (!activeSpaceId.value) throw new Error("An active Space is required");
  return {
    id: form.id || crypto.randomUUID(),
    spaceId: activeSpaceId.value,
    name: form.name.trim(),
    endpoint: form.endpoint.trim(),
    secretKey: form.secretKey.trim() || null,
    toolName: form.toolName.trim(),
    effect: form.effect,
    enabled,
  };
}

async function testConnection() {
  busy.value = true;
  error.value = undefined;
  diagnostic.value = undefined;
  try {
    diagnostic.value = await testExternalToolConnection(bindingRequest(false));
  } catch (reason) {
    error.value = String(reason);
  } finally {
    busy.value = false;
  }
}

function reconnect(binding: ExternalToolBinding) {
  Object.assign(form, {
    id: binding.id,
    name: binding.name,
    endpoint: binding.endpoint,
    secretKey: binding.secretKey ?? "",
    toolName: binding.toolName,
    effect: binding.effect,
  });
}

async function remove(bindingId: string) {
  await deleteExternalToolBinding(bindingId);
  await refresh();
}

async function disable(bindingId: string) {
  await disableExternalToolBinding(bindingId);
  await refresh();
}

watch(activeSpaceId, refresh);
onMounted(refresh);
</script>

<template>
  <section class="flex h-full min-h-0 flex-col bg-panel">
    <form class="space-y-2 border-b border-divider p-3" @submit.prevent="save">
      <div class="grid grid-cols-2 gap-2">
        <UiInput v-model="form.name" :placeholder="$t('spaces.assistant.externalToolName')" />
        <UiInput v-model="form.toolName" :placeholder="$t('spaces.assistant.externalToolMethod')" />
        <UiInput v-model="form.endpoint" class="col-span-2" placeholder="https://…/mcp" />
        <UiInput v-model="form.secretKey" :placeholder="$t('spaces.assistant.secretKeyReference')" />
        <UiSelect
          v-model="form.effect"
          :label="$t('spaces.assistant.externalToolEffect')"
          :options="effectOptions"
        />
      </div>
      <UiButton type="submit" size="sm" variant="primary" :disabled="busy">
        {{ $t("spaces.assistant.reviewAndSave") }}
      </UiButton>
      <UiButton type="button" size="sm" variant="ghost" :disabled="busy" @click="testConnection">
        {{ $t("spaces.assistant.testConnection") }}
      </UiButton>
      <p v-if="error" class="text-meta text-danger">{{ error }}</p>
      <pre
        v-if="diagnostic"
        class="max-h-48 overflow-auto rounded-control bg-control p-2 text-micro whitespace-pre-wrap"
        >{{ JSON.stringify(diagnostic, null, 2) }}</pre>
    </form>
    <div class="min-h-0 flex-1 overflow-y-auto p-2">
      <article
        v-for="binding in bindings"
        :key="binding.id"
        class="mb-2 rounded-control border border-border bg-raised p-2"
      >
        <div class="flex items-center gap-2">
          <strong class="min-w-0 flex-1 truncate text-label">{{ binding.name }}</strong>
          <UiBadge size="sm">{{ binding.effect }}</UiBadge>
          <UiBadge v-if="!binding.enabled" size="sm" variant="danger">Reconnect required</UiBadge>
        </div>
        <p class="truncate text-micro text-foreground-secondary">
          {{ binding.endpoint }} · {{ binding.toolName }}
        </p>
        <p class="mt-1 font-mono text-micro break-all text-foreground-tertiary">{{ binding.id }}</p>
        <p class="font-mono text-micro break-all text-foreground-tertiary">{{ binding.schemaFingerprint }}</p>
        <UiButton class="mt-1" size="sm" variant="dangerSubtle" @click="remove(binding.id)">
          {{ $t("common.delete") }}
        </UiButton>
        <UiButton v-if="binding.enabled" class="mt-1" size="sm" variant="ghost" @click="disable(binding.id)">
          {{ $t("spaces.assistant.disableConnection") }}
        </UiButton>
        <UiButton class="mt-1" size="sm" variant="ghost" @click="reconnect(binding)">
          {{ binding.enabled ? $t("spaces.assistant.reviewAgain") : $t("spaces.assistant.reconnect") }}
        </UiButton>
      </article>
    </div>
  </section>
</template>
