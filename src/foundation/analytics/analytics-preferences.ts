import { load as loadStore } from "@tauri-apps/plugin-store";
import { defineStore } from "pinia";
import { shallowRef } from "vue";

import { invokeCommand } from "../ipc/invoke-command";

type PersistedPreferences = {
  schemaVersion: 1;
  enabled: boolean;
  installationId: string | null;
  lastSentDate: string | null;
};

type AnalyticsDeliveryResult = "sent" | "not_configured" | "unavailable" | "disabled";

const STORE_FILE = "analytics-consent.json";
const STORE_KEY = "consent";
const INSTALLATION_ID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const UTC_DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/;

function parsePersistedPreferences(raw: unknown): PersistedPreferences | null {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) return null;

  const value = raw as Record<string, unknown>;
  if (value.schemaVersion !== 1 || typeof value.enabled !== "boolean") return null;
  if (!value.enabled) {
    return {
      schemaVersion: 1,
      enabled: false,
      installationId: null,
      lastSentDate: null,
    };
  }

  if (typeof value.installationId !== "string" || !INSTALLATION_ID_PATTERN.test(value.installationId)) {
    return null;
  }

  return {
    schemaVersion: 1,
    enabled: true,
    installationId: value.installationId,
    lastSentDate:
      typeof value.lastSentDate === "string" && UTC_DATE_PATTERN.test(value.lastSentDate)
        ? value.lastSentDate
        : null,
  };
}

export const useAnalyticsPreferencesStore = defineStore("analytics-preferences", () => {
  const enabled = shallowRef(false);
  const ready = shallowRef(false);
  let installationId: string | null = null;
  let lastSentDate: string | null = null;
  let persistence: Awaited<ReturnType<typeof loadStore>> | undefined;
  let attemptedThisRun = false;

  async function getPersistence() {
    persistence ??= await loadStore(STORE_FILE, { autoSave: false });
    return persistence;
  }

  async function persistPreferences(value: PersistedPreferences) {
    const store = await getPersistence();
    await store.set(STORE_KEY, value);
    await store.save();
  }

  function currentPreferences(): PersistedPreferences {
    return {
      schemaVersion: 1,
      enabled: enabled.value,
      installationId,
      lastSentDate,
    };
  }

  async function load() {
    try {
      const store = await getPersistence();
      const raw = await store.get<unknown>(STORE_KEY);
      if (raw === undefined || raw === null) {
        enabled.value = true;
        installationId = globalThis.crypto.randomUUID();
        lastSentDate = null;
        await persistPreferences(currentPreferences());
      } else {
        const preferences = parsePersistedPreferences(raw);
        if (preferences) {
          enabled.value = preferences.enabled;
          installationId = preferences.installationId;
          lastSentDate = preferences.lastSentDate;
        } else {
          // Unknown or damaged state must not silently opt an existing installation in.
          enabled.value = false;
          installationId = null;
          lastSentDate = null;
        }
      }
    } catch {
      enabled.value = false;
      installationId = null;
      lastSentDate = null;
    } finally {
      ready.value = true;
    }
  }

  async function setEnabled(next: boolean) {
    if (!ready.value) throw new Error("Usage analytics preferences have not been loaded.");

    const nextPreferences: PersistedPreferences = {
      schemaVersion: 1,
      enabled: next,
      installationId: next ? (installationId ?? globalThis.crypto.randomUUID()) : null,
      lastSentDate: next ? lastSentDate : null,
    };
    await persistPreferences(nextPreferences);
    enabled.value = nextPreferences.enabled;
    installationId = nextPreferences.installationId;
    lastSentDate = nextPreferences.lastSentDate;
    attemptedThisRun = false;
  }

  async function recordAppStarted(): Promise<AnalyticsDeliveryResult> {
    if (!ready.value || !enabled.value || !installationId) return "disabled";

    const today = new Date().toISOString().slice(0, 10);
    if (attemptedThisRun || lastSentDate === today) return "disabled";
    attemptedThisRun = true;

    try {
      const result = await invokeCommand<AnalyticsDeliveryResult>("plugin:desktop|send_analytics_event", {
        installationId,
      });
      if (result === "sent") {
        lastSentDate = today;
        await persistPreferences(currentPreferences());
      }
      return result;
    } catch {
      return "unavailable";
    }
  }

  return { enabled, ready, load, setEnabled, recordAppStarted };
});
