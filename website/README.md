# Knovum website

A static, bilingual product website for Knovum.

## Preview locally

From the repository root:

```bash
python3 -m http.server 4173 --directory website
```

Then open http://localhost:4173.

The site uses plain HTML, CSS and JavaScript so it can be hosted on GitHub Pages or any static host without a build step. Product screenshots are loaded from the repository's README assets. The primary download links point to the latest GitHub Release.

Languages: English and Simplified Chinese. The first visit follows the browser language; the selected language is remembered in local storage.
