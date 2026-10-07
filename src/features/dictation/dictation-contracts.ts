type DictationPhase = "idle" | "recording" | "transcribing" | "result" | "failed";

type DictationInsertion = "inserted" | "not_attempted" | "focus_changed" | "permission_denied" | "failed";

export interface DictationSnapshot {
  phase: DictationPhase;
  sessionId: string | null;
  startedAt: number | null;
  targetApplication: string | null;
  text: string | null;
  language: string | null;
  insertion: DictationInsertion | null;
  errorCode: string | null;
  errorMessage: string | null;
}
