# Knovum website

A standalone, dependency-free marketing website for Knovum. It intentionally lives in `website/` so it does not replace or interfere with the Vue/Tauri desktop application's root `index.html`.

## Run locally

From this directory:

```bash
python3 -m http.server 4174
```

Then open [http://localhost:4174](http://localhost:4174).

You can also serve this directory with any static file host. No build step or package installation is required.

## Structure

- `index.html` — page structure, SEO metadata, content, and external product links
- `styles.css` — responsive visual system and motion
- `script.js` — language switch, mobile navigation, scroll reveal, and FAQ behavior

The site reuses the product icon and screenshots already maintained in the repository under `public/` and `docs/assets/readme/`. Those assets are referenced from the repository's raw GitHub URLs so the website does not duplicate large binary files.

## Content and product claims

Product capabilities, local-first behavior, cloud-provider boundaries, and platform requirements are based on the repository README and `DESIGN.md`. Keep those sources in sync when product behavior changes.
