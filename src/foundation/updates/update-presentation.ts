import type { UpdateFailure } from "../../shared/contracts/generated/UpdateFailure";
import type { UpdateState } from "../../shared/contracts/generated/UpdateState";

export const UPDATE_STATUS_KEYS: Record<Exclude<UpdateState["status"], "failed">, string> = {
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

export const UPDATE_FAILURE_KEYS: Record<UpdateFailure, string> = {
  check: "updates.failure.check",
  download: "updates.failure.download",
  signature: "updates.failure.signature",
  install: "updates.failure.install",
};
