import type { ExportFormats } from "./formats";

/** Something that can be saved as a file, and the formats it admits. */
export interface ExportTarget {
  /** Identifies the listing, so formats are never reused across targets. */
  key: string;
  formats: () => Promise<ExportFormats>;
  /** Writes the file; `onStart` opens the work that can still be cancelled. */
  save: (format: string, onStart: (jobId: string) => void) => Promise<boolean>;
}
