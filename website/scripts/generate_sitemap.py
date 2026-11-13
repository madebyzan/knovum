#!/usr/bin/env python3
"""Generate canonical URLs, localized metadata, robots sitemap, and XML sitemap."""

from __future__ import annotations

import argparse
import re
import sys
import xml.etree.ElementTree as ET
from html import escape as escape_html
from pathlib import Path
from urllib.parse import urlsplit, urlunsplit

ROOT = Path(__file__).resolve().parents[1]
HTML_NAMESPACE = "http://www.sitemaps.org/schemas/sitemap/0.9"
XHTML_NAMESPACE = "http://www.w3.org/1999/xhtml"
ET.register_namespace("", HTML_NAMESPACE)
ET.register_namespace("xhtml", XHTML_NAMESPACE)


def parse_base_url(value: str) -> str:
    parsed = urlsplit(value)
    if parsed.scheme not in {"http", "https"} or not parsed.netloc:
        raise ValueError("--base-url must be an absolute http(s) URL")
    if parsed.username or parsed.password or parsed.query or parsed.fragment:
        raise ValueError("--base-url must not contain credentials, a query, or a fragment")
    return urlunsplit((parsed.scheme, parsed.netloc, parsed.path.rstrip("/"), "", ""))


def public_pages() -> list[Path]:
    return sorted(
        path for path in ROOT.rglob("*.html")
        if path.is_file() and path.name.lower() != "404.html"
    )


def relative_route(path: Path) -> str:
    relative = path.relative_to(ROOT).as_posix()
    if relative == "index.html":
        return ""
    if relative.endswith("/index.html"):
        return relative[: -len("index.html")]
    return relative


def public_url(base_url: str, path: Path) -> str:
    route = relative_route(path)
    return base_url + "/" + route.lstrip("/")


def is_chinese_page(path: Path) -> bool:
    return path.relative_to(ROOT).parts[0] == "zh"


def language_pair(path: Path) -> tuple[Path | None, Path | None]:
    relative = path.relative_to(ROOT)
    if relative.parts and relative.parts[0] == "zh":
        english = ROOT.joinpath(*relative.parts[1:])
        chinese = path
    else:
        english = path
        chinese = ROOT / "zh" / relative
    if english.is_file() and chinese.is_file():
        return english, chinese
    return None, None


def build_sitemap(base_url: str, pages: list[Path]) -> ET.Element:
    urlset = ET.Element(f"{{{HTML_NAMESPACE}}}urlset")
    for path in pages:
        entry = ET.SubElement(urlset, f"{{{HTML_NAMESPACE}}}url")
        ET.SubElement(entry, f"{{{HTML_NAMESPACE}}}loc").text = public_url(base_url, path)
        english, chinese = language_pair(path)
        if english and chinese:
            for language, target in (
                ("en", english),
                ("zh-CN", chinese),
                ("x-default", english),
            ):
                ET.SubElement(
                    entry,
                    f"{{{XHTML_NAMESPACE}}}link",
                    {
                        "rel": "alternate",
                        "hreflang": language,
                        "href": public_url(base_url, target),
                    },
                )
    return urlset


def upsert_meta(html: str, attribute: str, key: str, value: str, *, required: bool = False) -> str:
    pattern = re.compile(
        rf'<meta\s+{re.escape(attribute)}=["\']{re.escape(key)}["\']'
        rf'\s+content=["\'][^"\']*["\']\s*/?>',
        re.IGNORECASE,
    )
    replacement = (
        f'<meta {attribute}="{escape_html(key, quote=True)}" '
        f'content="{escape_html(value, quote=True)}">'
    )
    html, count = pattern.subn(lambda _: replacement, html)
    if count > 1:
        raise ValueError(f"duplicate meta {attribute}={key}")
    if count == 0:
        if required:
            raise ValueError(f"missing meta {attribute}={key}")
        if "</head>" not in html:
            raise ValueError("page has no closing head tag")
        html = html.replace("</head>", f"  {replacement}\n</head>", 1)
    return html


def configure_html(base_url: str, pages: list[Path]) -> None:
    canonical_pattern = re.compile(r'\s*<link\s+rel=["\']canonical["\'][^>]*>', re.I)
    alternate_pattern = re.compile(r'\s*<link\s+rel=["\']alternate["\']\s+hreflang=["\'][^"\']+["\'][^>]*>', re.I)
    og_url_pattern = re.compile(r'\s*<meta\s+property=["\']og:url["\'][^>]*>', re.I)

    for path in pages:
        html = path.read_text(encoding="utf-8")
        if re.search(r"<meta\s+<meta\b", html, re.IGNORECASE):
            raise ValueError(f"{path.relative_to(ROOT)} contains malformed nested meta markup")

        html = canonical_pattern.sub("", html)
        html = alternate_pattern.sub("", html)
        html = og_url_pattern.sub("", html)

        url = public_url(base_url, path)
        image_name = "social-card-zh.png" if is_chinese_page(path) else "social-card.png"
        image_url = f"{base_url}/assets/{image_name}"
        image_alt = (
            "Knovum 中文分享预览图：本地优先的知识工作空间，连接资料、研究与创作。"
            if is_chinese_page(path)
            else "Knovum social preview: a local-first workspace for connected knowledge, research, and creation."
        )
        locale = "zh_CN" if is_chinese_page(path) else "en_US"

        html = upsert_meta(html, "property", "og:image", image_url, required=True)
        html = upsert_meta(html, "name", "twitter:image", image_url, required=True)
        html = upsert_meta(html, "property", "og:image:alt", image_alt)
        html = upsert_meta(html, "name", "twitter:image:alt", image_alt)
        html = upsert_meta(html, "property", "og:locale", locale)
        html = upsert_meta(html, "property", "og:image:type", "image/png")
        html = upsert_meta(html, "property", "og:image:width", "1200")
        html = upsert_meta(html, "property", "og:image:height", "630")
        html = upsert_meta(html, "name", "twitter:card", "summary_large_image")

        tags = [
            f'<link rel="canonical" href="{escape_html(url, quote=True)}">',
            f'<meta property="og:url" content="{escape_html(url, quote=True)}">',
        ]
        english, chinese = language_pair(path)
        if english and chinese:
            tags.extend(
                [
                    f'<link rel="alternate" hreflang="en" href="{escape_html(public_url(base_url, english), quote=True)}">',
                    f'<link rel="alternate" hreflang="zh-CN" href="{escape_html(public_url(base_url, chinese), quote=True)}">',
                    f'<link rel="alternate" hreflang="x-default" href="{escape_html(public_url(base_url, english), quote=True)}">',
                ]
            )
        if "</head>" not in html:
            raise ValueError(f"{path.relative_to(ROOT)} has no closing head tag")
        html = html.replace("</head>", "  " + "\n  ".join(tags) + "\n</head>", 1)
        path.write_text(html, encoding="utf-8")


def configure_robots(base_url: str, robots_path: Path) -> None:
    content = robots_path.read_text(encoding="utf-8") if robots_path.exists() else "User-agent: *\nAllow: /\n"
    content = re.sub(r"^Sitemap:.*\n?", "", content, flags=re.I | re.M).rstrip()
    content += f"\n\nSitemap: {base_url}/sitemap.xml\n"
    robots_path.parent.mkdir(parents=True, exist_ok=True)
    robots_path.write_text(content, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-url", required=True, help="Public site URL, e.g. https://knovum.app")
    parser.add_argument("--output", type=Path, default=ROOT / "sitemap.xml")
    parser.add_argument("--robots-output", type=Path, default=ROOT / "robots.txt")
    parser.add_argument(
        "--apply-html",
        action="store_true",
        help="Write absolute canonical, social image, Open Graph, and hreflang metadata",
    )
    args = parser.parse_args()

    try:
        base_url = parse_base_url(args.base_url)
        pages = public_pages()
        if not pages:
            raise ValueError("no public HTML pages were found")
        sitemap = build_sitemap(base_url, pages)
        ET.indent(sitemap, space="  ")
        args.output.parent.mkdir(parents=True, exist_ok=True)
        ET.ElementTree(sitemap).write(args.output, encoding="utf-8", xml_declaration=True)
        configure_robots(base_url, args.robots_output)
        if args.apply_html:
            configure_html(base_url, pages)
    except (OSError, ValueError) as exc:
        print(f"site SEO generation failed: {exc}", file=sys.stderr)
        return 1

    print(f"Generated {args.output} with {len(pages)} canonical page URLs.")
    if args.apply_html:
        print(f"Applied canonical, social-image, and hreflang metadata to {len(pages)} HTML pages.")
    print(f"Updated sitemap directive in {args.robots_output}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
