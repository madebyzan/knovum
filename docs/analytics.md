# Usage analytics

Knovum has two independent analytics paths:

- The public website uses Cloudflare Web Analytics when the GitHub Actions variable `CLOUDFLARE_WEB_ANALYTICS_TOKEN` is configured. Without the token, the beacon is not loaded.
- The desktop app sends at most one `app_started` event per installation per UTC day when the Worker endpoint is configured. Desktop usage analytics is enabled by default on first launch. Users can disable it at any time in Settings → About; that opt-out is persisted across restarts.

On first launch, the desktop app creates and persists a random installation UUID. The event contains only that identifier, app version, platform, and event name. The native Rust host owns the HTTPS request; the WebView CSP is not widened. The Worker HMACs the UUID before persistence and stores only the digest, UTC day, version, platform, and event name. A unique database key deduplicates an installation's daily event. Daily rows are deleted after 90 days. Disabling analytics stops future requests and clears the local installation identifier.

No Spaces, node content, extracted text, file names or paths, search queries, prompts, generated outputs, API credentials, account identifiers, personal names, email addresses, or request IP addresses are written by the desktop Worker to D1. Cloudflare may process connection metadata under its own service terms. The event schema rejects extra fields and unsupported event types.

## Metrics

The D1 event table is the source of truth for desktop activity. The SQL in [cloudflare/analytics/queries/metrics.sql](../cloudflare/analytics/queries/metrics.sql) calculates:

- DAU: distinct installations active on each UTC date.
- WAU: distinct installations active in the current date and preceding 6 UTC dates.
- MAU: distinct installations active in the current date and preceding 29 UTC dates.
- First-seen installations and strict day-1/day-7/day-30 cohort retention within the 90-day retained dataset.

These are installation-based usage metrics, not account- or person-based user counts. The repository contains executable SQL for the Cloudflare D1 SQL console; it does not add a hosted analytics dashboard.

## Build configuration

The desktop endpoint is compiled into a release only when `KNOVUM_ANALYTICS_ENDPOINT` is supplied to the Tauri build environment. The release workflow reads it from the GitHub Actions repository variable of the same name. A missing or empty endpoint disables sending, even though the local preference defaults to enabled. The endpoint must be an HTTPS URL and should end in `/event`.

The website uses the GitHub Actions variable `CLOUDFLARE_WEB_ANALYTICS_TOKEN`; it is public site configuration, not a credential. An empty value leaves the beacon disabled.

Desktop analytics is enabled by default and has a Settings → About opt-out. Review the notice and applicable privacy requirements before enabling the feature in a public release.

See [cloudflare/analytics/README.md](../cloudflare/analytics/README.md) for Worker/D1 deployment, Free-plan requirements, retention, and aggregate SQL queries.
