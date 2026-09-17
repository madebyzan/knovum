import { formatBytes } from "../../../foundation/format/bytes";

export function formatByteSize(value: number | undefined): string | undefined {
  if (value === undefined) return undefined;
  return formatBytes(value);
}
