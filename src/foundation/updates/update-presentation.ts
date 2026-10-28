import type { UpdateFailure } from "../../shared/contracts/generated/UpdateFailure";
import type { UpdateState } from "../../shared/contracts/generated/UpdateState";
import { formatBytes } from "../format/bytes";

export const LABEL_KEY_BY_UPDATE_STATUS: Record<Exclude<UpdateState["status"], "failed">, string> = {
  disabled: "updates.status.disabled",
  idle: "updates.status.idle",
  checking: "updates.status.checking",
  current: "updates.status.current",
  available: "updates.status.available",
  downloading: "updates.status.downloading",
  verifying: "updates.status.verifying",
  ready: "updates.status.ready",
  installing: "updates.status.installing",
};

export const MESSAGE_KEY_BY_UPDATE_FAILURE: Record<UpdateFailure, string> = {
  check: "updates.failure.check",
  download: "updates.failure.download",
  signature: "updates.failure.signature",
  install: "updates.failure.install",
};

/** How far an update has come on its way in, read for the progress bar and the line beneath it. */
export interface UpdateTransfer {
  /** Bytes received out of `max`; `null` while the size is unknown, which sweeps the bar. */
  value: number | null;
  max: number;
  /** "12 MiB / 48 MiB", or what has arrived when the size is unknown. */
  amount: string | undefined;
  /** The share received, as the locale writes a percentage. */
  percent: string | undefined;
}

export function updateTransfer(state: UpdateState, locale: string): UpdateTransfer | undefined {
  // Checking the signature takes no measurable amount of work, so the bar only shows it is under way.
  if (state.status === "verifying") return { value: null, max: 1, amount: undefined, percent: undefined };
  if (state.status !== "downloading") return undefined;
  const { downloaded, total } = state;
  if (!total) return { value: null, max: 1, amount: formatBytes(downloaded), percent: undefined };
  const share = Math.min(downloaded / total, 1);
  return {
    value: downloaded,
    max: total,
    amount: `${formatBytes(downloaded)} / ${formatBytes(total)}`,
    percent: new Intl.NumberFormat(locale, { style: "percent", maximumFractionDigits: 0 }).format(
      Math.floor(share * 100) / 100,
    ),
  };
}

/**
 * Release notes as the update panel shows them. A release opens with its own title ("# Knovum
 * 0.0.3"), which the panel already names above the notes, so that leading title is left out.
 */
export function releaseNotesBody(notes: string): string {
  return notes.replace(/^\s*#\s[^\n]*\n*/u, "").trim();
}
