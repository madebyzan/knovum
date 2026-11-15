import { load as loadStore } from "@tauri-apps/plugin-store";
import { defineStore } from "pinia";
import { shallowRef } from "vue";

import { invokeCommand } from "../ipc/invoke-command";

type PersistedPreferences = {
  enabled: boolean;
  installationId: string;
  lastSentDate: string | null;
};

type AnalyticsDeliveryResult = "sent" | "not_configured" | "unavailable" | "disabled";

const STORE_FILE = "analytics-consent.json";
const STORE_KEY = "consent";

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

  async function persistPreferences(preferences: PersistedPreferences) {
    const store = await getPersistence();
    await store.set(STORE_KEY, preferences);
    await store.save();
  }

  async function load() {
    try {
      const store = await getPersistence();
      const preferences = await store.get<PersistedPreferences>(STORE_KEY);
      if (preferences?.enabled) {
        enabled.value = true;
        installationId = preferences.installationId;
        lastSentDate = preferences.lastSentDate;
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

    const store = await getPersistence();
    if (next) {
      const nextInstallationId = installationId ?? globalThis.crypto.randomUUID();
      await persistPreferences({
        enabled: true,
        installationId: nextInstallationId,
        lastSentDate,
      });
      installationId = nextInstallationId;
      enabled.value = true;
    } else {
      await store.delete(STORE_KEY);
      await store.save();
      enabled.value = false;
      installationId = null;
      lastSentDate = null;
    }
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
        await persistPreferences({
          enabled: true,
          installationId,
          lastSentDate,
        });
      }
      return result;
    } catch {
      return "unavailable";
    }
  }

  return { enabled, ready, load, setEnabled, recordAppStarted };
});
