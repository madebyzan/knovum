import { computed, onMounted, ref, shallowRef } from "vue";
import type { ZodIssue } from "zod";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  loadObjectStorageSettings,
  saveObjectStorageSettings,
  testObjectStorageConnection,
} from "../../../foundation/object-storage/object-storage-settings";
import { objectStorageSettingsSchema } from "../../../foundation/object-storage/schema/object-storage-settings";

export interface ObjectStorageDraft {
  endpoint: string;
  region: string;
  bucket: string;
  publicBaseUrl: string;
}

export type ObjectStorageField = keyof ObjectStorageDraft;

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
  const busy = shallowRef(false);
  const loadError = shallowRef("");
  const testing = shallowRef(false);
  const hasAccessKey = shallowRef(false);
  const hasSecretKey = shallowRef(false);
  const touched = ref<Partial<Record<ObjectStorageField, boolean>>>({});
  const issues = ref<Record<string, string>>({});

  const fieldErrors = computed(() => {
    const visible: Record<string, string> = {};
    for (const [field, message] of Object.entries(issues.value)) {
      if (touched.value[field as ObjectStorageField]) visible[field] = message;
    }
    return visible;
  });

  const canTest = computed(
    () => parseDraft(draft.value)?.success === true && hasAccessKey.value && hasSecretKey.value,
  );

  onMounted(async () => {
    try {
      const settings = await loadObjectStorageSettings();
      if (!settings) return;
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

  async function withBusyGuard(action: () => Promise<unknown>) {
    if (busy.value) return;
    busy.value = true;
    try {
      await action();
    } finally {
      busy.value = false;
    }
  }

  async function persist() {
    const parsed = parseDraft(draft.value);
    if (!parsed || !parsed.success) return false;
    await saveObjectStorageSettings(parsed.data);
    draft.value = {
      endpoint: parsed.data.endpoint,
      region: parsed.data.region,
      bucket: parsed.data.bucket,
      publicBaseUrl: parsed.data.publicBaseUrl ?? "",
    };
    return true;
  }

  async function saveField(field: ObjectStorageField) {
    touched.value = { ...touched.value, [field]: true };
    const parsed = parseDraft(draft.value);
    issues.value = parsed && !parsed.success ? extractIssues(parsed.error.issues) : {};
    if (!parsed || !parsed.success) return;
    await withBusyGuard(persist);
  }

  async function test() {
    await withBusyGuard(async () => {
      testing.value = true;
      try {
        if (await persist()) await testObjectStorageConnection();
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
    busy,
    loadError,
    testing,
    canTest,
    fieldErrors,
    saveField,
    test,
    onSecretChange,
  };
}
