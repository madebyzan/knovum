import { load as loadStore } from "@tauri-apps/plugin-store";
import { defineStore } from "pinia";
import { shallowRef } from "vue";

import { invokeCommand } from "../ipc/invoke-command";

type PersistedConsent = {
  schemaVersion: 1;
  enabled: boolean;
  installationId: string | null;
  lastSentDate: string | null;
};

type AnalyticsDeliveryResult = "sent" | "not_configured" | "unavailable" | "disabled";

const STORE_FILE = "analytics-consent.json";
const INSTALLATION_ID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const UTC_DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/;

function parsePersistedConsent(raw: unknown): PersistedConsent | null {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) return null;
  const value = raw as Record<string, unknown>;
  const id = value.installationId;
  if (
    value.schemaVersion !== 1 ||
    value.enabled !== true ||
    typeof id !== "string" ||
    !INSTALLATION_ID_PATTERN.test(id)
  ) {
    return null;
  }

  const date = value.lastSentDate;
  return {
    schemaVersion: 1,
    enabled: true,
    installationId: id,
    lastSentDate: typeof date === "string" && UTC_DATE_PATTERN.test(date) ? date : null,
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

  async function load() {
    try {
      const store = await getPersistence();
      const consent = parsePersistedConsent(await store.get<unknown>("consent"));
      if (consent) {
        enabled.value = consent.enabled;
        installationId = consent.installationId;
        lastSentDate = consent.lastSentDate;
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
    if (!ready.value) throw new Error("Analytics consent has not been loaded.");
    const nextInstallationId = next ? (installationId ?? globalThis.crypto.randomUUID()) : null;
    const nextLastSentDate = next ? lastSentDate : null;
    const value: PersistedConsent = {
      schemaVersion: 1,
      enabled: next,
      installationId: nextInstallationId,
      lastSentDate: nextLastSentDate,
    };
    const store = await getPersistence();
    await store.set("consent", value);
    await store.save();
    enabled.value = next;
    installationId = nextInstallationId;
    lastSentDate = nextLastSentDate;
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
        const store = await getPersistence();
        await store.set("consent", {
          schemaVersion: 1,
          enabled: true,
          installationId,
          lastSentDate: today,
        } satisfies PersistedConsent);
        await store.save();
        lastSentDate = today;
      }
      return result;
    } catch {
      return "unavailable";
    }
  }

  return { enabled, ready, load, setEnabled, recordAppStarted };
});
