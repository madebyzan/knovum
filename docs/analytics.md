# Usage analytics

Knovum has two independent analytics paths:

- The public website uses Cloudflare Web Analytics when the GitHub Actions variable CLOUDFLARE_WEB_ANALYTICS_TOKEN is configured. Without the token, the beacon is not loaded.
- The desktop app sends at most one app_started event per installation per UTC day when the Worker endpoint is configured. Usage analytics is enabled by default for fresh installations and can be disabled at any time in Settings → About. An explicit opt-out from an earlier version remains disabled after upgrading.

The desktop event contains only a random installation UUID, the app version, platform, and event name. The native Rust host owns the HTTPS request; the WebView CSP is not widened. The Worker HMACs the UUID before persistence and stores only the digest, UTC day, version, platform, and event name. Daily rows are deleted after 90 days. Disabling analytics clears the local installation identifier and stops future requests. Network or service failures never block startup or normal product work.

No Spaces, node content, extracted text, file names or paths, search queries, prompts, generated outputs, API credentials, account identifiers, personal names, email addresses, or request IP addresses are written by the desktop Worker to D1. Cloudflare may process connection metadata under its own service terms. The event schema rejects extra fields and unsupported event types.

## Build configuration

The desktop endpoint is compiled into a release only when KNOVUM_ANALYTICS_ENDPOINT is supplied to the Tauri build environment. The release workflow reads it from the GitHub Actions repository variable of the same name. A missing or empty endpoint disables sending even though the local preference defaults to enabled. The endpoint must be an HTTPS URL and should end in /event.

The website uses the GitHub Actions variable CLOUDFLARE_WEB_ANALYTICS_TOKEN; it is public site configuration, not a credential. An empty value leaves the beacon disabled.

The default-on setting is an explicit product choice, not a blanket legal conclusion. Confirm that the notice and opt-out model is appropriate for the jurisdictions where the app is distributed. Where prior opt-in is required, do not ship default-on reporting for that market.

See cloudflare/analytics/README.md for Worker/D1 deployment, Free-plan requirements, retention, and aggregate SQL queries.
