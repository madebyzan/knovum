# Usage analytics

Knovum has two independent analytics paths:

- The public website uses Cloudflare Web Analytics when the GitHub Actions variable CLOUDFLARE_WEB_ANALYTICS_TOKEN is configured. Without the token, the beacon is not loaded.
- The desktop app sends at most one app_started event per installation per UTC day when the Worker endpoint is configured. Usage analytics is disabled by default. Events are sent only after the user enables sharing in Settings → About, and sharing can be disabled at any time.

The desktop event contains only a random installation UUID, the app version, platform, and event name. The native Rust host owns the HTTPS request; the WebView CSP is not widened. The Worker HMACs the UUID before persistence and stores only the digest, UTC day, version, platform, and event name. Daily rows are deleted after 90 days. Disabling analytics clears the local installation identifier and stops future requests. Network or service failures never block startup or normal product work.

No Spaces, node content, extracted text, file names or paths, search queries, prompts, generated outputs, API credentials, account identifiers, personal names, email addresses, or request IP addresses are written by the desktop Worker to D1. Cloudflare may process connection metadata under its own service terms. The event schema rejects extra fields and unsupported event types.

## Build configuration

The desktop endpoint is compiled into a release only when KNOVUM_ANALYTICS_ENDPOINT is supplied to the Tauri build environment. The release workflow reads it from the GitHub Actions repository variable of the same name. A missing or empty endpoint disables sending, regardless of the local preference. The endpoint must be an HTTPS URL and should end in /event.

The website uses the GitHub Actions variable CLOUDFLARE_WEB_ANALYTICS_TOKEN; it is public site configuration, not a credential. An empty value leaves the beacon disabled.

Usage analytics requires an explicit user opt-in. Review the notice and applicable privacy requirements before enabling it in a release.

See cloudflare/analytics/README.md for Worker/D1 deployment, Free-plan requirements, retention, and aggregate SQL queries.
