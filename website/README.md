# Knovum website

The public-facing Knovum website is a standalone, dependency-free static site in this directory. It is intentionally separated from the Vue/Tauri desktop application's root entry point.

## Pages

- `index.html` — brand introduction and product overview
- `product.html` — detailed product capabilities and workflows
- `privacy.html` — local-first architecture and cloud-provider boundary
- `download.html` — supported platforms, installation notes, and source build
- `about.html` — product principles, architecture, and open-source community
- `guides/` — four long-tail educational guides plus a guide index, with matching static Simplified Chinese routes under `zh/`
- `404.html` — custom not-found page

## Run locally

From the repository root:

```bash
python3 -m http.server 4174 --directory website
```

Then open [http://localhost:4174](http://localhost:4174).

No package install or build step is required. Run the same lightweight checks used by CI with `node --check website/script.js` and `python3 website/scripts/check_site.py` from the repository root.

## Design and implementation

- Editorial visual system: warm paper, restrained forest green, sage and ochre accents, DM Sans, Instrument Serif, and DM Mono.
- Responsive navigation and layouts for desktop, tablet, and mobile.
- English and Simplified Chinese content on every first-party page. The selected language is remembered in local storage when available.
- Motion is progressive enhancement: scroll reveal, page entry, navigation, hover transitions, and reading progress. Reduced-motion preferences are respected.
- Semantic HTML, keyboard focus states, skip links, descriptive image alternative text, metadata, and structured software-application data.
- Real product screenshots and the maintained application icon are reused from the repository. The site deliberately does not present generated mockups as real product features.

See [SEO.md](SEO.md) for the page-to-intent map, GEO content principles, technical search checklist, and post-launch measurement plan.

## Content source of truth

Product capabilities, storage behavior, supported platforms, installer names, and toolchain requirements are grounded in the repository README, `DESIGN.md`, and the installation/system-architecture documentation. Keep the website in sync when those contracts change.

## Publishing

The transition deployment is configured in `.github/workflows/deploy-website.yml`. After merging to `main`, enable **Settings → Pages → Build and deployment → Source = GitHub Actions** once. The workflow generates the GitHub Pages sitemap, canonical URLs, Open Graph URLs, and language alternates, then publishes `website/` automatically to `https://madebyzan.github.io/knovum/`.

When the permanent domain is ready, update the workflow's `--base-url`, configure the custom domain in GitHub Pages, add the DNS records, and verify the new host in Google Search Console and Bing Webmaster Tools. The site remains static and dependency-free.
