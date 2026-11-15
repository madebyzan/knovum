const APP_STARTED_EVENT = "app_started";
const INSTALLATION_ID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const VERSION_PATTERN = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;
const ALLOWED_PLATFORMS = new Set(["macos", "windows"]);
const MAX_BODY_BYTES = 512;
const RETENTION_DAYS = 90;

function json(status, body) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json; charset=utf-8", "cache-control": "no-store" },
  });
}

function validatePayload(value) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return null;
  const keys = Object.keys(value).sort();
  if (keys.join(",") !== "appVersion,event,installationId,platform") return null;
  if (value.event !== APP_STARTED_EVENT) return null;
  if (typeof value.installationId !== "string" || !INSTALLATION_ID_PATTERN.test(value.installationId)) {
    return null;
  }
  if (typeof value.appVersion !== "string" || !VERSION_PATTERN.test(value.appVersion)) return null;
  if (typeof value.platform !== "string" || !ALLOWED_PLATFORMS.has(value.platform)) return null;
  return {
    installationId: value.installationId.toLowerCase(),
    event: value.event,
    appVersion: value.appVersion,
    platform: value.platform,
  };
}

async function hashInstallationId(installationId, salt) {
  const key = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(salt),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const digest = await crypto.subtle.sign("HMAC", key, new TextEncoder().encode(installationId));
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (request.method !== "POST" || url.pathname !== "/event") return json(404, { error: "not_found" });
    if (!env.DB || typeof env.INSTALLATION_HASH_SALT !== "string" || env.INSTALLATION_HASH_SALT.length < 32) {
      return json(503, { error: "analytics_unavailable" });
    }

    const contentLength = Number(request.headers.get("content-length") || "0");
    if (contentLength > MAX_BODY_BYTES) return json(413, { error: "payload_too_large" });

    let raw;
    try {
      const body = await request.text();
      if (new TextEncoder().encode(body).byteLength > MAX_BODY_BYTES) {
        return json(413, { error: "payload_too_large" });
      }
      raw = JSON.parse(body);
    } catch {
      return json(400, { error: "invalid_json" });
    }

    const event = validatePayload(raw);
    if (!event) return json(400, { error: "invalid_event" });

    try {
      const installationHash = await hashInstallationId(event.installationId, env.INSTALLATION_HASH_SALT);
      const activityDate = new Date().toISOString().slice(0, 10);
      await env.DB.prepare(
        "INSERT INTO desktop_daily_active (installation_hash, activity_date, event_name, app_version, platform) " +
          "VALUES (?1, ?2, ?3, ?4, ?5) " +
          "ON CONFLICT (installation_hash, activity_date, event_name) DO NOTHING",
      )
        .bind(installationHash, activityDate, event.event, event.appVersion, event.platform)
        .run();
      return new Response(null, { status: 204, headers: { "cache-control": "no-store" } });
    } catch {
      // Free-tier quota exhaustion or a transient database failure must never affect the desktop app.
      return json(503, { error: "analytics_unavailable" });
    }
  },

  async scheduled(_controller, env) {
    const cutoff = new Date(Date.now() - RETENTION_DAYS * 24 * 60 * 60 * 1000).toISOString().slice(0, 10);
    await env.DB.prepare("DELETE FROM desktop_daily_active WHERE activity_date < ?1").bind(cutoff).run();
  },
};
