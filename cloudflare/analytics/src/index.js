const APP_STARTED_EVENT = "app_started";
const EXPECTED_KEYS = "appVersion,event,installationId,platform";
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

async function readBoundedBody(request) {
  if (!request.body) return "";
  const decoder = new TextDecoder();
  let bytes = 0;
  let body = "";

  for await (const chunk of request.body) {
    bytes += chunk.byteLength;
    if (bytes > MAX_BODY_BYTES) return null;
    body += decoder.decode(chunk, { stream: true });
  }
  return body + decoder.decode();
}

function isRecord(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function validatePayload(value) {
  if (!isRecord(value)) return null;
  if (Object.keys(value).toSorted().join(",") !== EXPECTED_KEYS) return null;

  const { installationId, event, appVersion, platform } = value;
  if (event !== APP_STARTED_EVENT) return null;
  if (typeof installationId !== "string" || !INSTALLATION_ID_PATTERN.test(installationId)) return null;
  if (typeof appVersion !== "string" || !VERSION_PATTERN.test(appVersion)) return null;
  if (typeof platform !== "string" || !ALLOWED_PLATFORMS.has(platform)) return null;

  return { installationId: installationId.toLowerCase(), event, appVersion, platform };
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

async function parseEventRequest(request) {
  const contentLength = Number(request.headers.get("content-length") || "0");
  if (contentLength > MAX_BODY_BYTES) return { error: "payload_too_large", status: 413 };

  try {
    const body = await readBoundedBody(request);
    if (body === null) return { error: "payload_too_large", status: 413 };
    const event = validatePayload(JSON.parse(body));
    return event ? { event } : { error: "invalid_event", status: 400 };
  } catch {
    return { error: "invalid_json", status: 400 };
  }
}

async function persistEvent(event, env) {
  const installationHash = await hashInstallationId(event.installationId, env.INSTALLATION_HASH_SALT);
  const activityDate = new Date().toISOString().slice(0, 10);
  await env.DB.prepare(
    "INSERT INTO desktop_daily_active (installation_hash, activity_date, event_name, app_version, platform) " +
      "VALUES (?1, ?2, ?3, ?4, ?5) " +
      "ON CONFLICT (installation_hash, activity_date, event_name) DO NOTHING",
  )
    .bind(installationHash, activityDate, event.event, event.appVersion, event.platform)
    .run();
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (request.method !== "POST" || url.pathname !== "/event") return json(404, { error: "not_found" });
    if (!env.DB || typeof env.INSTALLATION_HASH_SALT !== "string" || env.INSTALLATION_HASH_SALT.length < 32) {
      return json(503, { error: "analytics_unavailable" });
    }

    const parsed = await parseEventRequest(request);
    if ("error" in parsed) return json(parsed.status, { error: parsed.error });

    try {
      await persistEvent(parsed.event, env);
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
