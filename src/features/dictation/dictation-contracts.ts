import type { ResourcePreparationProgress } from "../../shared/contracts/generated/ResourcePreparationProgress";

type DictationPhase = "idle" | "recording" | "preparing_resources" | "transcribing" | "result" | "failed";

type DictationInsertion = "inserted" | "not_attempted" | "focus_changed" | "permission_denied" | "failed";

export interface DictationSnapshot {
  phase: DictationPhase;
  resources: ResourcePreparationProgress | null;
  sessionId: string | null;
  startedAt: number | null;
  targetApplication: string | null;
  text: string | null;
  language: string | null;
  insertion: DictationInsertion | null;
  errorCode: string | null;
  errorMessage: string | null;
}
