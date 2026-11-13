#!/usr/bin/env python3
"""Validate the standalone Knovum website without third-party dependencies."""

from __future__ import annotations

from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
PAGES = tuple(
    path.relative_to(ROOT).as_posix()
    for path in sorted(ROOT.rglob("*.html"))
    if path.is_file()
)


class PageAudit(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.ids: list[str] = []
        self.links: list[str] = []
        self.images_without_alt: list[str] = []
        self.meta_names: set[str] = set()
        self.has_viewport = False
        self.has_language = False
        self.external_blank_without_rel: list[str] = []
        self.stylesheets: list[str] = []
        self.scripts: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attributes = dict(attrs)
        if tag == "html":
            self.has_language = bool(attributes.get("lang"))
        if tag == "meta":
            name = (attributes.get("name") or "").lower()
            if name:
                self.meta_names.add(name)
            if name == "viewport":
                self.has_viewport = True
        if attributes.get("id"):
            self.ids.append(attributes["id"] or "")
        if tag == "a" and attributes.get("href"):
            href = attributes["href"] or ""
            self.links.append(href)
            if attributes.get("target") == "_blank":
                rel = set((attributes.get("rel") or "").lower().split())
                if not {"noopener", "noreferrer"} & rel:
                    self.external_blank_without_rel.append(href)
        if tag == "img" and "alt" not in attributes:
            self.images_without_alt.append(attributes.get("src") or "(missing src)")
        if tag == "link" and "stylesheet" in (attributes.get("rel") or "").lower():
            if attributes.get("href"):
                self.stylesheets.append(attributes["href"] or "")
        if tag == "script" and attributes.get("src"):
            self.scripts.append(attributes["src"] or "")


def resolve_local_target(page: Path, href_path: str) -> Path:
    decoded_path = unquote(href_path)
    if decoded_path.startswith("/"):
        return (ROOT / decoded_path.lstrip("/")).resolve()
    return (page.parent / decoded_path).resolve()


def fail(message: str) -> None:
    raise SystemExit(f"website check failed: {message}")


def main() -> None:
    errors: list[str] = []
    if not PAGES:
        fail("no HTML pages were found")

    for name in PAGES:
        path = ROOT / name
        audit = PageAudit()
        try:
            audit.feed(path.read_text(encoding="utf-8"))
            audit.close()
        except (OSError, UnicodeError, ValueError) as exc:
            errors.append(f"{name}: could not parse HTML: {exc}")
            continue

        duplicates = sorted({item for item in audit.ids if audit.ids.count(item) > 1})
        if duplicates:
            errors.append(f"{name}: duplicate IDs: {', '.join(duplicates)}")
        if not audit.has_language:
            errors.append(f"{name}: missing html lang attribute")
        if not audit.has_viewport:
            errors.append(f"{name}: missing responsive viewport metadata")
        if "description" not in audit.meta_names:
            errors.append(f"{name}: missing meta description")
        if "robots" not in audit.meta_names:
            errors.append(f"{name}: missing robots meta")
        if audit.images_without_alt:
            errors.append(f"{name}: images missing alt text: {', '.join(audit.images_without_alt)}")
        if audit.external_blank_without_rel:
            errors.append(f"{name}: target=_blank links must include both noopener and noreferrer: {', '.join(audit.external_blank_without_rel)}")

        known_ids = set(audit.ids)
        for href in audit.links:
            parsed = urlsplit(href)
            if parsed.scheme or parsed.netloc or href.startswith(("mailto:", "tel:", "javascript:")):
                continue
            destination = resolve_local_target(path, parsed.path) if parsed.path else path.resolve()
            if not destination.is_relative_to(ROOT.resolve()):
                errors.append(f"{name}: link escapes website directory: {href}")
                continue
            if parsed.path and not destination.is_file():
                errors.append(f"{name}: missing local link target: {href}")
                continue
            if parsed.fragment:
                if parsed.path and destination != path.resolve():
                    target_audit = PageAudit()
                    target_audit.feed(destination.read_text(encoding="utf-8"))
                    target_ids = set(target_audit.ids)
                else:
                    target_ids = known_ids
                if unquote(parsed.fragment) not in target_ids:
                    errors.append(f"{name}: missing anchor target: {href}")

        for asset in audit.stylesheets + audit.scripts:
            parsed = urlsplit(asset)
            if parsed.scheme or parsed.netloc:
                continue
            asset_path = resolve_local_target(path, parsed.path)
            if not asset_path.is_relative_to(ROOT.resolve()) or not asset_path.is_file():
                errors.append(f"{name}: missing local asset: {asset}")

    css_path = ROOT / "styles.css"
    if css_path.is_file():
        css = css_path.read_text(encoding="utf-8")
        if css.count("{") != css.count("}"):
            errors.append("styles.css: unbalanced CSS braces")
    else:
        errors.append("missing styles.css")

    if errors:
        for error in errors:
            print(f"::error::{error}")
        fail(f"{len(errors)} issue(s) found")

    print(
        f"Website checks passed: {len(PAGES)} pages, local links, anchors, "
        "metadata, assets, and accessibility basics."
    )


if __name__ == "__main__":
    main()
