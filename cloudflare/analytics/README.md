# Knovum desktop analytics Worker

This endpoint receives at most one daily `app_started` event from each installation when the endpoint is configured. Desktop usage analytics is enabled by default on first launch. Users can disable it at any time in **Settings → About**; the opt-out is saved across restarts.

The Worker stores only an HMAC-SHA-256 digest of a random installation identifier, the UTC activity date, app version, platform, and event name. The original identifier is never written to D1. Rows older than 90 days are deleted by the daily Cron Trigger. The Worker does not write request IP addresses, request headers, note/file content, prompts, paths, credentials, account details, names, or email addresses to D1 or application logs. Cloudflare may process connection metadata under its own service terms.

The public website uses Cloudflare Web Analytics separately. Neither integration is active until its account-specific configuration is supplied.

## Deploy on the Free plan

Use a Cloudflare account on the **Workers Free** plan. This repository does not configure a Paid plan, enable billing, or automatically upgrade an account. Keep the account on Free; do not select a paid upgrade to increase quotas.

From this directory:

1. Install Node.js 22 or later, then authenticate Wrangler:

   ```sh
   npx wrangler@latest login
   ```

2. Create the D1 database:

   ```sh
   npx wrangler@latest d1 create knovum-analytics
   ```

   Copy the returned database ID into `d1_databases[0].database_id` in `wrangler.jsonc`.

3. Apply the schema and create a random HMAC salt:

   ```sh
   npx wrangler@latest d1 migrations apply knovum-analytics --remote
   openssl rand -hex 32 | npx wrangler@latest secret put INSTALLATION_HASH_SALT
   ```

   Keep the salt private. Do not put it in Git, an Actions variable, or the desktop application.

4. Deploy:

   ```sh
   npx wrangler@latest deploy
   ```

   The endpoint is the deployed Worker URL plus `/event`, for example `https://knovum-analytics.<your-account>.workers.dev/event`.

5. In the GitHub repository, open **Settings → Secrets and variables → Actions → Variables** and add `KNOVUM_ANALYTICS_ENDPOINT` with that HTTPS URL. The release workflow embeds it into desktop packages at build time. It is a public endpoint, not a credential. When configured, the desktop app starts sending daily events automatically unless the user has turned analytics off in Settings → About. If the variable is absent, no events are sent.

## Enable website analytics

1. In Cloudflare, open **Analytics & Logs → Web Analytics → Add a site** and add the hostname `madebyzan.github.io`.
2. Open **Manage site** and copy the token from the manual JavaScript snippet.
3. Add a GitHub Actions repository variable named `CLOUDFLARE_WEB_ANALYTICS_TOKEN` with that token. The GitHub Pages deployment workflow injects it into the generated site. Without the variable, the beacon script is not loaded.

The GitHub Pages site is not hosted on Cloudflare, so the manual Web Analytics beacon is used. No DNS change or Cloudflare proxy is required.

## Inspect desktop usage

Open the D1 database in the Cloudflare dashboard and use its SQL console. The ready-to-run reports are in [`queries/metrics.sql`](queries/metrics.sql):

- Daily trend: DAU, rolling 7-day WAU, rolling 30-day MAU, and first-seen installations for each UTC activity date.
- Cohort retention: strict day-1, day-7, and day-30 return rates grouped by the first activity date observed in the retained dataset.

These are installation-based metrics: one pseudonymous installation identifier represents one installation, not a verified person or account. Activity dates are UTC. Because raw daily records are retained for 90 days, all rolling metrics and cohort reports are limited to that retained window. "First-seen installations" means first observed within the retained data; it cannot distinguish a brand-new install from an older install that returns after more than 90 days away.

The desktop app does not currently include a hosted analytics dashboard. Run the SQL in the D1 console to inspect the figures.

## Free-tier guardrails

Cloudflare currently documents a Workers Free allowance of 100,000 requests per day and 10 ms CPU time per HTTP request. D1 Free currently includes 5 million rows read per day, 100,000 rows written per day, 500 MB per database, and 5 GB total storage per account. Daily limits reset at 00:00 UTC. On Free, D1 requests fail after a daily read/write quota is reached until the reset; this Worker responds with HTTP 503 and the desktop app continues normally.

The desktop app sends at most one request per enabled installation per UTC day, and D1 uses a conflict-safe no-op insert to keep the stored row unique. The database keeps a 90-day window. Monitor the account's Free quota and stored data. If a quota is reached, leave the service unavailable until the reset or reduce usage; do not upgrade to a Paid plan. Cloudflare can change plan limits, so re-check the official pricing and limits before deployment.

## Local checks

```sh
node --test test/index.test.mjs
```

The Worker has no npm runtime dependencies. Wrangler is invoked with `npx` only for account login, D1 migrations, and deployment.
