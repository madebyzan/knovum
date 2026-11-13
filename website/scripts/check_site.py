#!/usr/bin/env python3
"""Validate the standalone Knovum website without third-party dependencies."""

from __future__ import annotations

import argparse
import json
import re
import struct
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
        self.meta_properties: set[str] = set()
        self.meta_values: dict[str, list[str]] = {}
        self.canonical_urls: list[str] = []
        self.alternate_urls: dict[str, list[str]] = {}
        self.title_text: list[str] = []
        self.in_title = False
        self.json_ld_payloads: list[str] = []
        self.current_json_ld: list[str] = []
        self.in_json_ld = False
        self.h1_count = 0
        self.has_viewport = False
        self.has_language = False
        self.external_blank_without_rel: list[str] = []
        self.stylesheets: list[str] = []
        self.scripts: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attributes = dict(attrs)
        if tag == "html":
            self.has_language = bool(attributes.get("lang"))
        if tag == "title":
            self.in_title = True
        if tag == "h1":
            self.h1_count += 1
        if tag == "script" and (attributes.get("type") or "").lower() == "application/ld+json":
            self.in_json_ld = True
            self.current_json_ld = []
        if tag == "meta":
            name = (attributes.get("name") or "").lower()
            prop = (attributes.get("property") or "").lower()
            value = attributes.get("content") or ""
            if name:
                self.meta_names.add(name)
                self.meta_values.setdefault(name, []).append(value)
            if prop:
                self.meta_properties.add(prop)
                self.meta_values.setdefault(prop, []).append(value)
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
        if tag == "link":
            rel_tokens = set((attributes.get("rel") or "").lower().split())
            href = attributes.get("href") or ""
            if "stylesheet" in rel_tokens and href:
                self.stylesheets.append(href)
            if "canonical" in rel_tokens and href:
                self.canonical_urls.append(href)
            if "alternate" in rel_tokens and attributes.get("hreflang") and href:
                self.alternate_urls.setdefault(attributes["hreflang"] or "", []).append(href)
        if tag == "script" and attributes.get("src"):
            self.scripts.append(attributes["src"] or "")

    def handle_data(self, data: str) -> None:
        if self.in_title:
            self.title_text.append(data)
        if self.in_json_ld:
            self.current_json_ld.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag == "title":
            self.in_title = False
        if tag == "script" and self.in_json_ld:
            self.json_ld_payloads.append("".join(self.current_json_ld))
            self.current_json_ld = []
            self.in_json_ld = False


def resolve_local_target(page: Path, href_path: str) -> Path:
    decoded_path = unquote(href_path)
    if decoded_path.startswith("/"):
        return (ROOT / decoded_path.lstrip("/")).resolve()
    return (page.parent / decoded_path).resolve()


def route_for(path: Path) -> str:
    relative = path.relative_to(ROOT).as_posix()
    if relative == "index.html":
        return ""
    if relative.endswith("/index.html"):
        return relative[: -len("index.html")]
    return relative


def fail(message: str) -> None:
    raise SystemExit(f"website check failed: {message}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--require-seo-output",
        action="store_true",
        help="Require generated canonical, hreflang, and absolute share metadata",
    )
    parser.add_argument(
        "--expected-base-url",
        help="Expected absolute public URL used by the SEO build step",
    )
    args = parser.parse_args()
    expected_base = args.expected_base_url.rstrip("/") if args.expected_base_url else None

    errors: list[str] = []
    if not PAGES:
        fail("no HTML pages were found")

    for name in PAGES:
        path = ROOT / name
        source = path.read_text(encoding="utf-8")
        audit = PageAudit()
        try:
            audit.feed(source)
            audit.close()
        except (OSError, UnicodeError, ValueError) as exc:
            errors.append(f"{name}: could not parse HTML: {exc}")
            continue

        if re.search(r"<meta\s+<meta\b", source, re.IGNORECASE):
            errors.append(f"{name}: malformed nested <meta> tag")
        duplicates = sorted({item for item in audit.ids if audit.ids.count(item) > 1})
        if duplicates:
            errors.append(f"{name}: duplicate IDs: {', '.join(duplicates)}")
        if not audit.has_language:
            errors.append(f"{name}: missing html lang attribute")
        if not audit.has_viewport:
            errors.append(f"{name}: missing responsive viewport metadata")
        if len(re.findall(r"<title\b", source, re.IGNORECASE)) != 1 or not "".join(audit.title_text).strip():
            errors.append(f"{name}: missing or duplicate page title")
        if "description" not in audit.meta_names or not any(v.strip() for v in audit.meta_values.get("description", [])):
            errors.append(f"{name}: missing or empty meta description")
        if "robots" not in audit.meta_names:
            errors.append(f"{name}: missing robots meta")
        if audit.h1_count != 1:
            errors.append(f"{name}: expected exactly one h1, found {audit.h1_count}")
        if audit.images_without_alt:
            errors.append(f"{name}: images missing alt text: {', '.join(audit.images_without_alt)}")
        if audit.external_blank_without_rel:
            errors.append(
                f"{name}: target=_blank links must include noopener or noreferrer: "
                + ", ".join(audit.external_blank_without_rel)
            )

        is_noindex = any("noindex" in value.lower() for value in audit.meta_values.get("robots", []))
        if not is_noindex:
            required_properties = {"og:title", "og:description", "og:image", "og:image:alt", "og:image:width", "og:image:height"}
            required_names = {"twitter:card", "twitter:title", "twitter:description", "twitter:image"}
            missing_properties = sorted(required_properties - audit.meta_properties)
            missing_names = sorted(required_names - audit.meta_names)
            if missing_properties:
                errors.append(f"{name}: missing Open Graph metadata: {', '.join(missing_properties)}")
            if missing_names:
                errors.append(f"{name}: missing Twitter metadata: {', '.join(missing_names)}")
            if audit.meta_values.get("og:image:width") != ["1200"] or audit.meta_values.get("og:image:height") != ["630"]:
                errors.append(f"{name}: social image dimensions must be declared as 1200x630")
            og_images = audit.meta_values.get("og:image", [])
            twitter_images = audit.meta_values.get("twitter:image", [])
            if len(og_images) != 1 or og_images != twitter_images:
                errors.append(f"{name}: Open Graph and Twitter image URLs must match exactly")
            for image_url in og_images:
                parsed_image = urlsplit(image_url)
                if parsed_image.scheme not in {"http", "https"} or not parsed_image.netloc:
                    errors.append(f"{name}: social image URL must be absolute: {image_url}")
                expected_file = "social-card-zh.png" if name.startswith("zh/") else "social-card.png"
                if not parsed_image.path.endswith("/assets/" + expected_file):
                    errors.append(f"{name}: expected localized social image {expected_file}")

        for payload in audit.json_ld_payloads:
            try:
                json.loads(payload)
            except json.JSONDecodeError as exc:
                errors.append(f"{name}: invalid JSON-LD structured data: {exc}")

        if args.require_seo_output:
            if not expected_base:
                errors.append("--expected-base-url is required with --require-seo-output")
            else:
                if not is_noindex:
                    expected_url = expected_base + "/" + route_for(path)
                    if audit.canonical_urls != [expected_url]:
                        errors.append(f"{name}: canonical must equal {expected_url}")
                    if audit.meta_values.get("og:url") != [expected_url]:
                        errors.append(f"{name}: og:url must equal canonical URL")
                    expected_image = expected_base + (
                        "/assets/social-card-zh.png" if name.startswith("zh/") else "/assets/social-card.png"
                    )
                    if audit.meta_values.get("og:image") != [expected_image]:
                        errors.append(f"{name}: social image must use the configured site base URL")
                    route = route_for(path).removeprefix("zh/")
                    expected_alternates = {
                        "en": expected_base + "/" + route,
                        "zh-CN": expected_base + "/zh/" + route,
                        "x-default": expected_base + "/" + route,
                    }
                    for language, expected in expected_alternates.items():
                        if audit.alternate_urls.get(language) != [expected]:
                            errors.append(f"{name}: hreflang {language} must point to {expected}")

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
        if "@media (prefers-reduced-motion:reduce)" not in css and "@media (prefers-reduced-motion: reduce)" not in css:
            errors.append("styles.css: missing reduced-motion support")
        if "@media (max-width:" not in css:
            errors.append("styles.css: missing responsive breakpoint")

        def relative_luminance(hex_color: str) -> float:
            channels = [int(hex_color[index:index + 2], 16) / 255 for index in (1, 3, 5)]
            linear = [
                channel / 12.92 if channel <= 0.04045 else ((channel + 0.055) / 1.055) ** 2.4
                for channel in channels
            ]
            return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]

        def contrast_ratio(foreground: str, background: str) -> float:
            first, second = relative_luminance(foreground), relative_luminance(background)
            return (max(first, second) + 0.05) / (min(first, second) + 0.05)

        for variable in ("--muted", "--accent-text"):
            match = re.search(rf"{re.escape(variable)}\s*:\s*(#[0-9a-fA-F]{{6}})", css)
            if not match:
                errors.append(f"styles.css: missing accessible color token {variable}")
                continue
            for surface in ("#f2f0e9", "#e8e6dd", "#e0dfd6"):
                if contrast_ratio(match.group(1), surface) < 4.5:
                    errors.append(f"styles.css: {variable} contrast is below WCAG AA on {surface}")
    else:
        errors.append("missing styles.css")

    if args.require_seo_output:
        for filename in ("social-card.png", "social-card-zh.png"):
            image_path = ROOT / "assets" / filename
            if not image_path.is_file():
                errors.append(f"missing rendered social image: assets/{filename}")
                continue
            image = image_path.read_bytes()
            if len(image) < 24 or image[:8] != b"\x89PNG\r\n\x1a\n":
                errors.append(f"assets/{filename}: not a valid PNG")
                continue
            width, height = struct.unpack(">II", image[16:24])
            if (width, height) != (1200, 630):
                errors.append(f"assets/{filename}: expected 1200x630, found {width}x{height}")

    if errors:
        for error in errors:
            print(f"::error::{error}")
        fail(f"{len(errors)} issue(s) found")

    print(
        f"Website checks passed: {len(PAGES)} pages, valid metadata, local links, "
        "anchors, assets, structured data, and accessibility basics."
    )


if __name__ == "__main__":
    main()
