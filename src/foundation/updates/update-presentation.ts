import type { UpdateFailure } from "../../shared/contracts/generated/UpdateFailure";
import type { UpdateState } from "../../shared/contracts/generated/UpdateState";

export const updateStatusKeys: Record<Exclude<UpdateState["status"], "failed">, string> = {
  disabled: "settings.updates.status.disabled",
  idle: "settings.updates.status.idle",
  checking: "settings.updates.status.checking",
  current: "settings.updates.status.current",
  available: "settings.updates.status.available",
  downloading: "settings.updates.status.downloading",
  verifying: "settings.updates.status.verifying",
  ready: "settings.updates.status.ready",
  installing: "settings.updates.status.installing",
};

export const updateFailureKeys: Record<UpdateFailure, string> = {
  check: "settings.updates.failure.check",
  download: "settings.updates.failure.download",
  signature: "settings.updates.failure.signature",
  install: "settings.updates.failure.install",
};
