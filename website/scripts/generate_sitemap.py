#!/usr/bin/env python3
"""Generate canonical URLs, language alternates, robots sitemap, and XML sitemap."""

from __future__ import annotations

import argparse
import re
import sys
import xml.etree.ElementTree as ET
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
            ET.SubElement(
                entry,
                f"{{{XHTML_NAMESPACE}}}link",
                {
                    "rel": "alternate",
                    "hreflang": "en",
                    "href": public_url(base_url, english),
                },
            )
            ET.SubElement(
                entry,
                f"{{{XHTML_NAMESPACE}}}link",
                {
                    "rel": "alternate",
                    "hreflang": "zh-CN",
                    "href": public_url(base_url, chinese),
                },
            )
            ET.SubElement(
                entry,
                f"{{{XHTML_NAMESPACE}}}link",
                {
                    "rel": "alternate",
                    "hreflang": "x-default",
                    "href": public_url(base_url, english),
                },
            )
    return urlset


def configure_html(base_url: str, pages: list[Path]) -> None:
    canonical_pattern = re.compile(r'\s*<link\s+rel=["\']canonical["\'][^>]*>', re.I)
    alternate_pattern = re.compile(r'\s*<link\s+rel=["\']alternate["\']\s+hreflang=["\'][^"\']+["\'][^>]*>', re.I)
    og_url_pattern = re.compile(r'\s*<meta\s+property=["\']og:url["\'][^>]*>', re.I)

    for path in pages:
        html = path.read_text(encoding="utf-8")
        html = canonical_pattern.sub("", html)
        html = alternate_pattern.sub("", html)
        html = og_url_pattern.sub("", html)

        url = public_url(base_url, path)
        tags = [f'<link rel="canonical" href="{url}">', f'<meta property="og:url" content="{url}">']
        english, chinese = language_pair(path)
        if english and chinese:
            tags.extend(
                [
                    f'<link rel="alternate" hreflang="en" href="{public_url(base_url, english)}">',
                    f'<link rel="alternate" hreflang="zh-CN" href="{public_url(base_url, chinese)}">',
                    f'<link rel="alternate" hreflang="x-default" href="{public_url(base_url, english)}">',
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
    parser.add_argument("--base-url", required=True, help="Production origin, e.g. https://knovum.app")
    parser.add_argument("--output", type=Path, default=ROOT / "sitemap.xml")
    parser.add_argument("--robots-output", type=Path, default=ROOT / "robots.txt")
    parser.add_argument(
        "--apply-html",
        action="store_true",
        help="Write absolute canonical, Open Graph, and hreflang links into each HTML page",
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
        print(f"Applied canonical and hreflang metadata to {len(pages)} HTML pages.")
    print(f"Updated sitemap directive in {args.robots_output}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
