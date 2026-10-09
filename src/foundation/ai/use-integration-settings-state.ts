import { useDebounceFn } from "@vueuse/core";
import { cloneDeep } from "es-toolkit";
import pLimit from "p-limit";
import { computed, onScopeDispose, ref, shallowRef, watch } from "vue";

interface IntegrationStateOptions<T> {
  initial: T;
  load: () => Promise<T>;
  save: (value: T) => Promise<void>;
  reset: () => Promise<void>;
}

export function useIntegrationSettingsState<T>(options: IntegrationStateOptions<T>) {
  const value = ref<T>(cloneDeep(options.initial));
  const loaded = shallowRef(false);
  const loading = shallowRef(false);
  const saving = shallowRef(false);
  const loadError = shallowRef<unknown>();
  const saveError = shallowRef<unknown>();
  let loadPromise: Promise<void> | undefined;
  let requestedRevision = 0;
  let savedRevision = 0;
  let pendingSave: Promise<void> | undefined;
  let pendingReset: Promise<void> | undefined;
  const serialize = pLimit(1);

  async function load() {
    loaded.value = false;
    loading.value = true;
    loadError.value = undefined;
    try {
      await serialize(async () => {
        value.value = await options.load();
        requestedRevision = 0;
        savedRevision = 0;
        saveError.value = undefined;
        loaded.value = true;
      });
    } catch (error) {
      loadError.value = error;
    } finally {
      loading.value = false;
    }
  }

  function ensureLoaded() {
    if (pendingReset) return pendingReset;
    if (loaded.value) return Promise.resolve();
    loadPromise ??= load().finally(() => {
      loadPromise = undefined;
    });
    return loadPromise;
  }

  async function drain() {
    const revision = requestedRevision;
    saving.value = true;
    saveError.value = undefined;
    try {
      await options.save(cloneDeep(value.value));
      savedRevision = revision;
    } catch (error) {
      saveError.value = error;
      return;
    } finally {
      saving.value = false;
    }
    if (savedRevision < requestedRevision) await drain();
  }

  function persist() {
    if (pendingSave) return pendingSave;
    if (!loaded.value) return Promise.resolve();
    if (savedRevision === requestedRevision) return Promise.resolve();
    pendingSave = serialize(drain).finally(() => {
      pendingSave = undefined;
    });
    return pendingSave;
  }

  async function flush(): Promise<void> {
    await pendingReset;
    await loadPromise;
    if (!loaded.value) {
      if (savedRevision < requestedRevision)
        throw loadError.value ?? new Error("Integration settings are not loaded");
      return;
    }
    await persist();
    if (saveError.value) throw saveError.value;
    if (savedRevision < requestedRevision) await flush();
  }

  const scheduleSave = useDebounceFn(() => void persist(), 350, { maxWait: 1200 });
  onScopeDispose(scheduleSave.cancel);
  watch(
    value,
    () => {
      if (!loaded.value || loading.value || pendingReset) return;
      requestedRevision += 1;
      void scheduleSave();
    },
    { deep: true, flush: "sync" },
  );

  function reset(): Promise<void> {
    if (pendingReset) return pendingReset;
    const wasLoaded = loaded.value;
    loaded.value = false;
    loading.value = true;
    pendingReset = serialize(async () => {
      try {
        await options.reset();
        value.value = cloneDeep(options.initial);
        requestedRevision = 0;
        savedRevision = 0;
        loadError.value = undefined;
        saveError.value = undefined;
        loaded.value = true;
      } catch (error) {
        loaded.value = wasLoaded;
        loadError.value = error;
        throw error;
      } finally {
        loading.value = false;
      }
    }).finally(() => {
      pendingReset = undefined;
    });
    return pendingReset;
  }

  function reload() {
    if (pendingReset) return pendingReset;
    loaded.value = false;
    return ensureLoaded();
  }

  return {
    value,
    ready: computed(() => loaded.value && !loading.value),
    loading,
    saving,
    loadError,
    saveError,
    ensureLoaded,
    reload,
    flush,
    reset,
  };
}
