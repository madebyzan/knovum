export type CodePreviewKind = "html" | "svg";

const HTML_LANGS = new Set(["html", "htm", "xhtml"]);
const HTML_EXTENSION = /\.(?:html?|xhtml)$/iu;
const SVG_EXTENSION = /\.svg$/iu;

export function resolveCodePreviewKind(language?: string, fileName?: string): CodePreviewKind | undefined {
  const normalized = language?.trim().toLowerCase() ?? "";
  if (HTML_LANGS.has(normalized) || (fileName && HTML_EXTENSION.test(fileName))) return "html";
  if (normalized === "svg" || (fileName && SVG_EXTENSION.test(fileName))) return "svg";
  return undefined;
}
