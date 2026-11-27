import type { BundledLanguage, Highlighter, ThemedToken } from "shiki";

const COLOR_VALUE = /^#[0-9a-fA-F]{3,8}$/u;
const LIGHT_THEME = "github-light-default";
const DARK_THEME = "github-dark-default";

let highlighterPromise: Promise<{ highlighter: Highlighter; languages: ReadonlySet<string> }> | undefined;
const loadedLanguages = new Set<string>();

async function getHighlighter() {
  highlighterPromise ??= (async () => {
    const [shiki, jsEngine] = await Promise.all([import("shiki"), import("shiki/engine/javascript")]);
    const highlighter = await shiki.createHighlighter({
      themes: [LIGHT_THEME, DARK_THEME],
      langs: [],
      engine: jsEngine.createJavaScriptRegexEngine({ forgiving: true }),
    });
    return { highlighter, languages: new Set(Object.keys(shiki.bundledLanguages)) };
  })().catch((error: unknown) => {
    highlighterPromise = undefined;
    throw error;
  });
  return highlighterPromise;
}

function escapeHtml(text: string): string {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function tokenStyle(token: ThemedToken): string {
  const declarations: string[] = [];
  if (token.color && COLOR_VALUE.test(token.color)) declarations.push(`color:${token.color}`);
  const style = token.fontStyle ?? 0;
  if (style & 1) declarations.push("font-style:italic");
  if (style & 2) declarations.push("font-weight:600");
  if (style & 4) declarations.push("text-decoration:underline");
  return declarations.join(";");
}

function renderTokenLine(tokens: readonly ThemedToken[]): string {
  if (!tokens.length) return " ";
  return tokens
    .map((token) => {
      const escaped = escapeHtml(token.content);
      const style = tokenStyle(token);
      return style ? `<span style="${style}">${escaped}</span>` : escaped;
    })
    .join("");
}

export interface CodeTokenSpan {
  readonly start: number;
  readonly end: number;
  readonly style: string;
}

async function codeTokens(
  code: string,
  normalizedLanguage: string,
  dark: boolean,
): Promise<readonly (readonly ThemedToken[])[] | undefined> {
  const { highlighter, languages } = await getHighlighter();
  if (!languages.has(normalizedLanguage)) return undefined;
  if (!loadedLanguages.has(normalizedLanguage)) {
    await highlighter.loadLanguage(normalizedLanguage as BundledLanguage);
    loadedLanguages.add(normalizedLanguage);
  }
  const result = highlighter.codeToTokens(code, {
    lang: normalizedLanguage as BundledLanguage,
    theme: dark ? DARK_THEME : LIGHT_THEME,
  });
  return result.tokens;
}

/** Tokenizes code into escaped per-line HTML spans using Shiki's JS RegExp engine. */
export async function tokenizeCodeLines(
  code: string,
  normalizedLanguage: string,
  dark: boolean,
): Promise<readonly string[] | undefined> {
  const tokens = await codeTokens(code, normalizedLanguage, dark);
  return tokens?.map(renderTokenLine);
}

/** Tokenizes code into UTF-16 offset style spans for ProseMirror inline decorations. */
export async function tokenizeCodeSpans(
  code: string,
  normalizedLanguage: string,
  dark: boolean,
): Promise<readonly CodeTokenSpan[] | undefined> {
  const lines = await codeTokens(code, normalizedLanguage, dark);
  if (!lines) return undefined;
  const spans: CodeTokenSpan[] = [];
  for (const line of lines) {
    for (const token of line) {
      if (!token.content.length) continue;
      const style = tokenStyle(token);
      if (!style) continue;
      spans.push({ start: token.offset, end: token.offset + token.content.length, style });
    }
  }
  return spans;
}
