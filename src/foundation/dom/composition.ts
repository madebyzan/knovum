/**
 * Whether a key press belongs to an input method still composing text: the Enter that confirms a
 * candidate must not also submit. Safari reports that Enter with `isComposing` already false, so
 * the legacy composition key code is checked too.
 */
export function isComposing(event: KeyboardEvent) {
  return event.isComposing || event.keyCode === 229;
}
