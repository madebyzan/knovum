import { onMounted, ref, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  loadObjectStorageSettings,
  removeObjectStorageConfiguration,
  saveObjectStorageSettings,
  testObjectStorageConnection,
} from "../../../foundation/object-storage/object-storage-settings";
import { objectStorageSettingsSchema } from "../../../foundation/object-storage/schema";

export interface ObjectStorageDraft {
  endpoint: string;
  region: string;
  bucket: string;
  publicBaseUrl: string;
}

const EMPTY_DRAFT: ObjectStorageDraft = {
  endpoint: "",
  region: "",
  bucket: "",
  publicBaseUrl: "",
};

export function useObjectStorageForm() {
  const draft = ref<ObjectStorageDraft>({ ...EMPTY_DRAFT });
  const configured = shallowRef(false);
  const busy = shallowRef(false);
  const loadError = shallowRef("");

  onMounted(async () => {
    try {
      const settings = await loadObjectStorageSettings();
      if (!settings) return;
      configured.value = true;
      draft.value = {
        endpoint: settings.endpoint,
        region: settings.region,
        bucket: settings.bucket,
        publicBaseUrl: settings.publicBaseUrl ?? "",
      };
    } catch (error) {
      loadError.value = getErrorMessage(error);
    }
  });

  async function guarded(action: () => Promise<void>) {
    if (busy.value) return;
    busy.value = true;
    try {
      await action();
    } finally {
      busy.value = false;
    }
  }

  async function save() {
    await guarded(async () => {
      const parsed = objectStorageSettingsSchema.parse({
        endpoint: draft.value.endpoint,
        region: draft.value.region,
        bucket: draft.value.bucket,
        publicBaseUrl: draft.value.publicBaseUrl.trim() || null,
      });
      await saveObjectStorageSettings(parsed);
      configured.value = true;
      draft.value = {
        endpoint: parsed.endpoint,
        region: parsed.region,
        bucket: parsed.bucket,
        publicBaseUrl: parsed.publicBaseUrl ?? "",
      };
    });
  }

  async function remove() {
    await guarded(async () => {
      await removeObjectStorageConfiguration();
      configured.value = false;
      draft.value = { ...EMPTY_DRAFT };
    });
  }

  function test() {
    return guarded(() => testObjectStorageConnection());
  }

  return { draft, configured, busy, loadError, save, remove, test };
}
