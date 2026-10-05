import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { PreparedVoiceReference } from "../contracts/generated/PreparedVoiceReference";
import type { PrepareVoiceReferenceRequest } from "../contracts/generated/PrepareVoiceReferenceRequest";

export function authorizeVoiceReferenceSource(sourcePath: string) {
  return invokeCommand<string>("plugin:spaces|authorize_voice_reference_source", { sourcePath });
}
export function prepareVoiceReference(request: PrepareVoiceReferenceRequest) {
  return invokeCommand<PreparedVoiceReference>("plugin:spaces|prepare_voice_reference", { request });
}
