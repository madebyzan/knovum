const EXTERNAL_LINK = /^(?:https?|mailto):/iu;

/**
 * Destinations the window hands to the system: a web page or a mail address. The window itself only
 * ever shows the app, so any other destination — a path, a relative URL — leads nowhere from a link.
 */
export function isExternalLink(href: string): boolean {
  return EXTERNAL_LINK.test(href);
}
