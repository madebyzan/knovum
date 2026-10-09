const MAX_OUTPUT_NAME_CODE_POINTS = 120;

export function workflowOutputNamesValid(names: readonly string[]) {
  const normalized = names.map((name) => name.trim());
  return (
    normalized.length > 0 &&
    normalized.every((name) => name.length > 0 && Array.from(name).length <= MAX_OUTPUT_NAME_CODE_POINTS) &&
    new Set(normalized).size === normalized.length
  );
}
