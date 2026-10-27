/**
 * Where following a link leads. The window only ever shows the app, so a link never navigates it:
 * - `external`: a web page or a mail address, handed to the system;
 * - `local`: a file path, which only a surface that knows what its paths name can follow — a
 *   conversation resolves its Agent's paths to what they became;
 * - `undefined`: nowhere — an in-page anchor, another scheme.
 */
export type LinkDestination = "external" | "local";

const EXTERNAL = /^(?:https?|mailto):/iu;
const SCHEME = /^[a-z][a-z\d+.-]*:/iu;
// `C:\…` reads as a scheme; rendered Markdown percent-encodes its backslash.
const WINDOWS_DRIVE = /^[a-z]:(?:[\\/]|%5c)/iu;

export function linkDestination(href: string): LinkDestination | undefined {
  if (EXTERNAL.test(href)) return "external";
  if (!href || href.startsWith("#") || href.startsWith("//")) return undefined;
  return WINDOWS_DRIVE.test(href) || !SCHEME.test(href) ? "local" : undefined;
}
