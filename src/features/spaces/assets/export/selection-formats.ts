import type { ExportFormats } from "./formats";

export const DEFAULT_EXPORT_FORMAT = "default";

/** Common conversions belong only to a homogeneous selection. */
export function selectionExportFormats(
  kinds: readonly string[],
  offers: readonly ExportFormats[],
): ExportFormats {
  if (!offers.length || offers.some(({ formats }) => !formats.length))
    return { formats: [], unavailable: offers.find(({ formats }) => !formats.length)?.unavailable ?? null };
  const homogeneous = kinds.every((kind) => kind === kinds[0]);
  const common = homogeneous
    ? offers[0]!.formats.filter((format) => offers.every(({ formats }) => formats.includes(format)))
    : [];
  return { formats: [DEFAULT_EXPORT_FORMAT, ...common], unavailable: null };
}
