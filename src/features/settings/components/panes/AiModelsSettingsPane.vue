<script setup lang="ts">
import { Cloud, Cpu } from "@lucide/vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { storeToRefs } from "pinia";
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from "reka-ui";
import { computed, onMounted, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import { useCreationLibraryStore } from "../../../../foundation/ai/creation-library-store";
import { loadProviderDescriptors, type ProviderDescriptor } from "../../../../foundation/ai/providers";
import { useProtectedExecutionTargets } from "../../use-protected-execution-targets";
import AiConnectionsSettings from "../AiConnectionsSettings.vue";
import SettingsLoadFailure from "../SettingsLoadFailure.vue";
import SpaceRuntimeSettings from "../SpaceRuntimeSettings.vue";

const { t } = useI18n();
const aiStore = useAiSettingsStore();
const creationStore = useCreationLibraryStore();
const { settings, ready: aiReady, loadError, saveError } = storeToRefs(aiStore);
const { ready: creationReady, loadError: creationLoadError } = storeToRefs(creationStore);
const providers = shallowRef<ProviderDescriptor[]>([]);
const catalogError = shallowRef<unknown>();
const catalogReady = shallowRef(false);
const activeSection = shallowRef("providers");
const sections = computed(() => [
  { value: "providers", label: t("settings.ai.navigationProviders"), icon: Cloud },
  { value: "onDevice", label: t("settings.ai.navigationOnDevice"), icon: Cpu },
]);
const failure = computed(() =>
  loadError.value
    ? { source: "ai" as const, cause: loadError.value, canReset: true }
    : creationLoadError.value
      ? { source: "creation" as const, cause: creationLoadError.value, canReset: true }
      : catalogError.value
        ? { source: "catalog" as const, cause: catalogError.value, canReset: false }
        : undefined,
);
const ready = computed(() => aiReady.value && creationReady.value && catalogReady.value);
const { protectedConnectionIds, protectedModelTargets } = useProtectedExecutionTargets();

watch(saveError, (error) => {
  if (error) toast.error(`${t("settings.ai.integrationSaveFailed")}: ${String(error)}`);
});

async function load() {
  catalogReady.value = false;
  catalogError.value = undefined;
  await Promise.all([aiStore.ensureLoaded(), creationStore.ensureLoaded()]);
  try {
    providers.value = await loadProviderDescriptors();
  } catch (error) {
    catalogError.value = error;
  } finally {
    catalogReady.value = true;
  }
}

async function resetSection() {
  const confirmed = await ask(t("settings.general.resetSectionConfirmation"), {
    title: t("settings.general.resetSection"),
    kind: "warning",
    okLabel: t("settings.general.resetSection"),
    cancelLabel: t("common.cancel"),
  });
  if (!confirmed) return;
  if (failure.value?.source === "creation") await creationStore.reset();
  else await aiStore.reset();
}

onMounted(load);
</script>

<template>
  <SettingsLoadFailure
    v-if="failure"
    :message="String(failure.cause)"
    :can-reset="failure.canReset"
    @retry="load"
    @reset="resetSection"
  />
  <TabsRoot v-else v-model="activeSection" :class="!ready && 'pointer-events-none opacity-60'">
    <p class="mb-4 max-w-2xl text-body text-foreground-secondary">{{ t("settings.ai.description") }}</p>
    <TabsList :aria-label="t('settings.ai.title')" class="mb-5 inline-flex rounded-control bg-control p-0.5">
      <TabsTrigger
        v-for="section in sections"
        :key="section.value"
        :value="section.value"
        class="flex h-(--knovum-control-height-md) items-center gap-1.5 rounded-control px-3 text-label text-foreground-secondary transition-colors outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring data-[state=active]:bg-raised data-[state=active]:text-foreground data-[state=active]:shadow-control"
      >
        <component :is="section.icon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
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
    </TabsContent>
    <TabsContent value="onDevice" class="outline-none"><SpaceRuntimeSettings /></TabsContent>
  </TabsRoot>
</template>
