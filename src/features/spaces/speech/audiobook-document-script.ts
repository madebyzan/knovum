import MarkdownIt from "markdown-it";

import type { AudiobookSource } from "../contracts/generated/AudiobookSource";
import type { ImportedAudiobookScript } from "./audiobook-script";

function parseXml(text: string) {
  const document = new DOMParser().parseFromString(text, "application/xml");
  if (document.getElementsByTagName("parsererror").length) throw new Error("EPUB metadata is malformed XML");
  return document;
}
function scriptFromMarkup(
  parts: readonly { title: string; markup: string }[],
  role: string,
): ImportedAudiobookScript {
  const chapters: ImportedAudiobookScript["chapters"] = [];
  for (const part of parts) {
    const document = new DOMParser().parseFromString(part.markup, "text/html");
    document.querySelectorAll("script,style,nav,header,footer").forEach((element) => element.remove());
    document.querySelectorAll("br").forEach((element) => element.replaceWith("\n"));
    let chapter = { title: part.title.slice(0, 200), paragraphs: [] as { role: string; text: string }[] };
    const blocks = [...document.querySelectorAll("h1,h2,p,li,blockquote,pre")].filter(
      (element) => !element.parentElement?.closest("li,blockquote,pre"),
    );
    for (const block of blocks) {
      const text = block.textContent?.trim();
      if (!text) continue;
      if (block.matches("h1,h2")) {
        if (chapter.paragraphs.length) chapters.push(chapter);
        chapter = { title: [...text].slice(0, 200).join(""), paragraphs: [] };
      } else {
        chapter.paragraphs.push({ role, text });
      }
    }
    if (chapter.paragraphs.length) chapters.push(chapter);
  }
  if (!chapters.length) throw new Error("Document contains no readable audiobook paragraphs");
  return { schemaVersion: 1, chapters };
}
export function audiobookDocumentScript(
  source: AudiobookSource,
  title: string,
  role: string,
): ImportedAudiobookScript {
  if (source.kind === "text") {
    const markup = new MarkdownIt({ html: false, breaks: false }).render(source.text);
    return scriptFromMarkup([{ title, markup }], role);
  }
  const entries = new Map(source.entries.map((entry) => [entry.path, entry.text]));
  if (entries.size !== source.entries.length) throw new Error("EPUB contains duplicate document paths");
  const container = entries.get("META-INF/container.xml");
  if (!container) throw new Error("EPUB container is missing");
  const packagePath = parseXml(container)
    .getElementsByTagNameNS("urn:oasis:names:tc:opendocument:xmlns:container", "rootfile")[0]
    ?.getAttribute("full-path");
  const packageText = packagePath ? entries.get(packagePath) : undefined;
  if (!packagePath || !packageText) throw new Error("EPUB package is missing");
  const document = parseXml(packageText);
  const manifest = new Map(
    [...document.getElementsByTagNameNS("http://www.idpf.org/2007/opf", "item")].map((item) => [
      item.getAttribute("id"),
      item,
    ]),
  );
  const parts = [...document.getElementsByTagNameNS("http://www.idpf.org/2007/opf", "itemref")]
    .filter((item) => item.getAttribute("linear") !== "no")
    .map((item, index) => {
      const entry = manifest.get(item.getAttribute("idref"));
      const href = entry?.getAttribute("href");
      if (!href || !["application/xhtml+xml", "text/html"].includes(entry?.getAttribute("media-type") ?? ""))
        throw new Error("EPUB spine contains an unsupported document");
      const url = new URL(href, `https://book.invalid/${packagePath}`);
      if (url.origin !== "https://book.invalid")
        throw new Error("EPUB spine references an external resource");
      const markup = entries.get(decodeURIComponent(url.pathname.slice(1)));
      if (!markup) throw new Error("EPUB spine document is missing");
      return { title: `${title} ${index + 1}`, markup };
    });
  return scriptFromMarkup(parts, role);
}
