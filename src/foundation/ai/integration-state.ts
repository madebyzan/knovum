import { useDebounceFn } from "@vueuse/core";
import { cloneDeep } from "es-toolkit";
import { computed, ref, shallowRef, watch } from "vue";

interface IntegrationStateOptions<T> {
  initial: T;
  load: () => Promise<T>;
  save: (value: T) => Promise<void>;
  reset: () => Promise<void>;
}

export function createIntegrationState<T>(options: IntegrationStateOptions<T>) {
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

  async function load() {
    loaded.value = false;
    loading.value = true;
    loadError.value = undefined;
    try {
      value.value = await options.load();
      loaded.value = true;
    } catch (error) {
      loadError.value = error;
    } finally {
      loading.value = false;
    }
  }

  function ensureLoaded() {
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
      await options.save(value.value);
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
    if (savedRevision === requestedRevision) return Promise.resolve();
    pendingSave = drain().finally(() => {
      pendingSave = undefined;
    });
    return pendingSave;
  }

  async function flush(): Promise<void> {
    await persist();
    if (saveError.value) throw saveError.value;
    if (savedRevision < requestedRevision) await flush();
  }

  const scheduleSave = useDebounceFn(() => void persist(), 350, { maxWait: 1200 });
  watch(
    value,
    () => {
      if (!loaded.value) return;
      requestedRevision += 1;
      void scheduleSave();
    },
    { deep: true, flush: "sync" },
  );

  async function reset() {
    loaded.value = false;
    await options.reset();
    value.value = cloneDeep(options.initial);
    requestedRevision = 0;
    savedRevision = 0;
    loaded.value = true;
  }

  return {
    value,
    ready: computed(() => loaded.value && !loading.value),
    loading,
    saving,
    loadError,
    saveError,
    ensureLoaded,
    reload: load,
    flush,
    reset,
  };
}
