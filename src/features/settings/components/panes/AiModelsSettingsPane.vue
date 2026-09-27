<script setup lang="ts">
import { Cloud, Cpu } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { ask } from "@tauri-apps/plugin-dialog";
import { storeToRefs } from "pinia";
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from "reka-ui";
import { computed, onMounted, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import { useCreationLibraryStore } from "../../../../foundation/ai/creation-library-store";
import { loadProviderDescriptors } from "../../../../foundation/ai/providers";
import { useProtectedExecutionTargets } from "../../use-protected-execution-targets";
import AiConnectionsSettings from "../AiConnectionsSettings.vue";
import FalImageLayerSettings from "../FalImageLayerSettings.vue";
import SettingsLoadFailure from "../SettingsLoadFailure.vue";
import SpaceRuntimeSettings from "../SpaceRuntimeSettings.vue";

const PROVIDER_CATALOG_KEY = ["integrations", "provider-descriptors"] as const;

const { t } = useI18n();
const aiStore = useAiSettingsStore();
const creationStore = useCreationLibraryStore();
const { settings, ready: aiReady, loadError, saveError } = storeToRefs(aiStore);
const { ready: creationReady, loadError: creationLoadError } = storeToRefs(creationStore);
const catalog = useQuery({
  queryKey: PROVIDER_CATALOG_KEY,
  queryFn: loadProviderDescriptors,
  staleTime: 30_000,
});
const providers = computed(() => catalog.data.value ?? []);
const activeSection = shallowRef("providers");
const sections = computed(() => [
  { value: "providers", label: t("settings.ai.navigation.providers"), icon: Cloud },
  { value: "onDevice", label: t("settings.ai.navigation.onDevice"), icon: Cpu },
]);
const failure = computed(() =>
  loadError.value
    ? { source: "ai" as const, cause: loadError.value, canReset: true }
    : creationLoadError.value
      ? { source: "creation" as const, cause: creationLoadError.value, canReset: true }
      : catalog.error.value
        ? { source: "catalog" as const, cause: catalog.error.value, canReset: false }
        : undefined,
);
const ready = computed(() => aiReady.value && creationReady.value && !catalog.isPending.value);
const { protectedConnectionIds, protectedModelTargets } = useProtectedExecutionTargets();

watch(saveError, (error) => {
  if (error) toast.error(`${t("settings.ai.integrationSaveFailed")}: ${String(error)}`);
});

async function hydrateStores() {
  await Promise.all([aiStore.ensureLoaded(), creationStore.ensureLoaded()]);
}

async function reload() {
  await Promise.all([hydrateStores(), catalog.refetch()]);
}

async function resetSection() {
  const confirmed = await ask(t("settings.general.reset.confirmation"), {
    title: t("settings.general.reset.action"),
    kind: "warning",
    okLabel: t("settings.general.reset.action"),
    cancelLabel: t("common.cancel"),
  });
  if (!confirmed) return;
  if (failure.value?.source === "creation") await creationStore.reset();
  else await aiStore.reset();
}

onMounted(hydrateStores);
</script>

<template>
  <SettingsLoadFailure
    v-if="failure"
    :message="String(failure.cause)"
    :can-reset="failure.canReset"
    @retry="reload"
    @reset="resetSection"
  />
  <TabsRoot v-else v-model="activeSection" :class="!ready && 'pointer-events-none opacity-60'">
    <p class="mb-4 max-w-2xl text-body-sm text-fg-secondary">{{ t("settings.ai.description") }}</p>
    <TabsList
      :aria-label="t('settings.ai.title')"
      class="mb-5 inline-flex rounded-md bg-surface-control p-0.5"
    >
      <TabsTrigger
        v-for="section in sections"
        :key="section.value"
        :value="section.value"
        class="flex h-(--knovum-control-height-md) items-center gap-1.5 rounded-md px-3 text-label-md text-fg-secondary transition-colors outline-none hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus data-[state=active]:bg-surface-raised data-[state=active]:text-fg-primary data-[state=active]:shadow-control"
      >
        <component :is="section.icon" class="size-3.5" :stroke-width="1.7" />
        {{ section.label }}
      </TabsTrigger>
    </TabsList>

    <TabsContent value="providers" class="grid gap-5 outline-none">
      <AiConnectionsSettings
        v-model="settings.connections"
        :providers="providers"
        :protected-connection-ids="protectedConnectionIds"
        :protected-model-targets="protectedModelTargets"
      />
      <FalImageLayerSettings />
    </TabsContent>
    <TabsContent value="onDevice" class="outline-none"><SpaceRuntimeSettings /></TabsContent>
  </TabsRoot>
</template>
