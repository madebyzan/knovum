# Optional usage analytics

Knovum has two independent, opt-in-by-configuration measurement paths:

- The static website can load Cloudflare Web Analytics when the public site token is configured in GitHub Actions. If the token is absent, no beacon script is inserted.
- Desktop usage analytics is off by default. A user must enable **Settings → About → Anonymous usage analytics**. Once enabled, Knovum sends at most one `app_started` event per UTC day, on the first app launch that day; both the local consent store and the Worker enforce daily deduplication.

The desktop event contains only a random installation UUID, the app version, platform, and event name. The native Rust host owns the HTTPS request; the WebView CSP is not widened. The Worker HMACs the UUID before persistence and stores only the digest, UTC day, version, platform, and event name. Daily rows are deleted after 90 days. Disabling consent clears the local UUID and prevents future requests. Network or service failures never block startup or normal product work.

No Spaces, node content, extracted text, file names or paths, search queries, prompts, generated outputs, API credentials, account identifiers, personal names, email addresses, or IP addresses are written by the desktop Worker to D1. The event schema rejects extra fields and unsupported event types.

## Build configuration

The desktop endpoint is compiled into a release only when `KNOVUM_ANALYTICS_ENDPOINT` is supplied to the Tauri build environment. The release workflow reads it from the GitHub Actions repository variable of the same name. A missing or empty endpoint disables sending at runtime. The endpoint must be an HTTPS URL and should end in `/event`.

The website uses the GitHub Actions variable `CLOUDFLARE_WEB_ANALYTICS_TOKEN`; it is public site configuration, not a credential. An empty value leaves the beacon disabled.

See [cloudflare/analytics/README.md](../cloudflare/analytics/README.md) for Worker/D1 deployment, Free-plan requirements, retention, and aggregate SQL queries.
