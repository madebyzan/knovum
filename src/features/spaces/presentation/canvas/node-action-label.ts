export const nodeActionLabelClass =
  "inline-block overflow-hidden whitespace-nowrap transition-[max-width,opacity] duration-150 ease-standard motion-reduce:transition-none";

export function nodeActionLabelState(compact: boolean) {
  return compact ? "max-w-0 opacity-0" : "max-w-56 opacity-100";
}
