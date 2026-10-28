/**
 * The zoom below which a node is drawn as its overview instead of mounting its content.
 *
 * Below it body text renders under about six pixels, so a mounted card shows nothing its overview does
 * not — while mounting one costs a component tree, rich-text rendering and media decoding per node.
 * Gating content on legibility bounds that work by the screen instead of by the Space: at this zoom the
 * screen holds a bounded number of cards however many the Space contains.
 */
export const NODE_CONTENT_MIN_ZOOM = 0.4;

export function showsNodeContent(zoom: number) {
  return zoom >= NODE_CONTENT_MIN_ZOOM;
}
