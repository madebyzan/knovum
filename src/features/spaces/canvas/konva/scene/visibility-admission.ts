/**
 * Cards admitted per animation frame while the camera moves.
 *
 * Mounting node content is the expensive half of becoming visible - a card builds DOM and renders its
 * payload - so a pan that reveals a dense region must not mount everything inside a single frame. The
 * lead overscan gives roughly two samples of runway, which is far more than the steady rate at which a
 * pan reveals content, so this budget only clips bursts.
 */
export const CONTENT_MOUNT_BUDGET = 4;

export interface AdmittedVisibility {
  ids: ReadonlySet<string>;
  /** Whether the target still holds ids that the budget could not admit yet. */
  pending: boolean;
}

/**
 * Moves `admitted` towards `target` by at most `budget` additions.
 *
 * Discards are applied in full: unmounting frees frame time, and keeping an invisible card alive would
 * defeat culling. Returns `undefined` when the admitted set already matches, so a caller can leave its
 * published value untouched and render nothing.
 */
export function admitVisibleIds(
  admitted: ReadonlySet<string>,
  target: ReadonlySet<string>,
  budget: number,
): AdmittedVisibility | undefined {
  const ids = new Set<string>();
  for (const id of admitted) if (target.has(id)) ids.add(id);
  let added = 0;
  for (const id of target) {
    if (added >= budget) break;
    if (ids.has(id)) continue;
    ids.add(id);
    added += 1;
  }
  if (added === 0 && ids.size === admitted.size) return undefined;
  return { ids, pending: ids.size < target.size };
}
