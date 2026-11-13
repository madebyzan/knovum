# Knovum website

The public-facing Knovum website is a standalone, dependency-free static site in this directory. It is intentionally separated from the Vue/Tauri desktop application's root entry point.

## Pages

- `index.html` — brand introduction and product overview
- `product.html` — detailed product capabilities and workflows
- `privacy.html` — local-first architecture and cloud-provider boundary
- `download.html` — supported platforms, installation notes, and source build
- `about.html` — product principles, architecture, and open-source community
- `404.html` — custom not-found page

## Run locally

From the repository root:

```bash
python3 -m http.server 4174 --directory website
```

Then open [http://localhost:4174](http://localhost:4174).

No package install or build step is required.

## Design and implementation

- Editorial visual system: warm paper, restrained forest green, sage and ochre accents, DM Sans, Instrument Serif, and DM Mono.
- Responsive navigation and layouts for desktop, tablet, and mobile.
- English and Simplified Chinese content on every first-party page. The selected language is remembered in local storage when available.
- Motion is progressive enhancement: scroll reveal, page entry, navigation, hover transitions, and reading progress. Reduced-motion preferences are respected.
- Semantic HTML, keyboard focus states, skip links, descriptive image alternative text, metadata, and structured software-application data.
- Real product screenshots and the maintained application icon are reused from the repository. The site deliberately does not present generated mockups as real product features.

## Content source of truth

Product capabilities, storage behavior, supported platforms, installer names, and toolchain requirements are grounded in the repository README, `DESIGN.md`, and the installation/system-architecture documentation. Keep the website in sync when those contracts change.

## Publishing

The directory can be deployed as a static site without a compilation step. Point a static host's publish/output directory at `website/`. A production domain, hosting provider, and DNS are intentionally not assumed here.
