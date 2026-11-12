import { computed, onMounted, ref, shallowRef } from "vue";
import type { ZodIssue } from "zod";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  loadObjectStorageSettings,
  OBJECT_STORAGE_ACCESS_KEY_ID,
  OBJECT_STORAGE_SECRET_ACCESS_KEY,
  removeObjectStorageConfiguration,
  saveObjectStorageSettings,
  testObjectStorageConnection,
} from "../../../foundation/object-storage/object-storage-settings";
import { objectStorageSettingsSchema } from "../../../foundation/object-storage/schema/object-storage-settings";
import { isSecretConfigured } from "../../../foundation/secrets/secret-store";

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

function extractIssues(issues: readonly ZodIssue[]) {
  const errors: Record<string, string> = {};
  for (const issue of issues) {
    const key = String(issue.path[0] ?? "");
    if (key && !errors[key]) errors[key] = issue.message;
  }
  return errors;
}

function parseDraft(draft: ObjectStorageDraft) {
  const endpoint = draft.endpoint.trim();
  const region = draft.region.trim();
  const bucket = draft.bucket.trim();
  const publicBaseUrl = draft.publicBaseUrl.trim();

  if (!endpoint && !region && !bucket && !publicBaseUrl) return null;

  return objectStorageSettingsSchema.safeParse({
    endpoint,
    region,
    bucket,
    publicBaseUrl: publicBaseUrl || null,
  });
}

export function useObjectStorageEditing() {
  const draft = ref<ObjectStorageDraft>({ ...EMPTY_DRAFT });
  const configured = shallowRef(false);
  const busy = shallowRef(false);
  const loadError = shallowRef("");
  const testing = shallowRef(false);
  const hasAccessKey = shallowRef(false);
  const hasSecretKey = shallowRef(false);
  const fieldErrors = ref<Record<string, string>>({});

  async function checkCredentials() {
    try {
      const [accessKeyConfigured, secretKeyConfigured] = await Promise.all([
        isSecretConfigured(OBJECT_STORAGE_ACCESS_KEY_ID),
        isSecretConfigured(OBJECT_STORAGE_SECRET_ACCESS_KEY),
      ]);
      hasAccessKey.value = accessKeyConfigured;
      hasSecretKey.value = secretKeyConfigured;
    } catch {
      // ignore
    }
  }

  onMounted(async () => {
    try {
      await checkCredentials();
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

  const canTest = computed(() => configured.value && hasAccessKey.value && hasSecretKey.value);
  const hasAnySecret = computed(() => hasAccessKey.value || hasSecretKey.value);

  async function withBusyGuard(action: () => Promise<void>) {
    if (busy.value) return;
    busy.value = true;
    try {
      await action();
    } finally {
      busy.value = false;
    }
  }

  async function saveField() {
    const parsed = parseDraft(draft.value);
    if (!parsed) {
      fieldErrors.value = {};
      return;
    }
    if (!parsed.success) {
      fieldErrors.value = extractIssues(parsed.error.issues);
      return;
    }

    fieldErrors.value = {};
    await withBusyGuard(async () => {
      await saveObjectStorageSettings(parsed.data);
      configured.value = true;
      draft.value = {
        endpoint: parsed.data.endpoint,
        region: parsed.data.region,
        bucket: parsed.data.bucket,
        publicBaseUrl: parsed.data.publicBaseUrl ?? "",
      };
    });
  }

  async function remove() {
    await withBusyGuard(async () => {
      await removeObjectStorageConfiguration();
      configured.value = false;
      hasAccessKey.value = false;
      hasSecretKey.value = false;
      draft.value = { ...EMPTY_DRAFT };
      fieldErrors.value = {};
    });
  }

  async function test() {
    await withBusyGuard(async () => {
      testing.value = true;
      try {
        await testObjectStorageConnection();
      } finally {
        testing.value = false;
      }
    });
  }

  function onSecretChange(key: "accessKeyId" | "secretAccessKey", isConfigured: boolean) {
    if (key === "accessKeyId") hasAccessKey.value = isConfigured;
    else hasSecretKey.value = isConfigured;
  }

  return {
    draft,
    configured,
    busy,
    loadError,
    testing,
    canTest,
    hasAnySecret,
    fieldErrors,
    saveField,
    remove,
    test,
    onSecretChange,
  };
}
