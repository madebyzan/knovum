import type { Editor } from "@tiptap/vue-3";

import { admitAiRequirements } from "../../../foundation/ai/ai-readiness-admission";
import { transformEditorTextStream } from "../../../foundation/ai/editor-transform";
import { translateTextStream, type TranslationTarget } from "../../../foundation/ai/translation";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { EditorContextReference } from "../../../shared/contracts/generated/EditorContextReference";
import type { EditorTransformAction } from "../../../shared/contracts/generated/EditorTransformAction";
import type { RoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";
import type { TaskStreamEvent } from "../../../shared/contracts/generated/TaskStreamEvent";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import { extractEditorDocumentContext } from "./tiptap-ai-context";
import type { RunningAction } from "./tiptap-writing-catalog";
import type { EditorWritingJob, WritingSessionState } from "./tiptap-writing-session-state";
import { markReasoning, normalizeGeneratedMarkdown, type TransformSource } from "./tiptap-writing-text";

export interface SessionRunSpec {
  readonly instance: Editor;
  readonly state: WritingSessionState;
  readonly actionLabel: RunningAction;
  readonly purpose: Extract<RoutingPurpose, "writing" | "translation">;
  readonly source: TransformSource;
  readonly onError: (message: string) => void;
  readonly executor: (job: EditorWritingJob) => Promise<string>;
}

export async function executeWritingSession(spec: SessionRunSpec): Promise<void> {
  const { instance, state } = spec;
  const job = state.beginJob(instance, spec.actionLabel, spec.source);
  const isCurrent = () => state.getActiveJob() === job && !instance.isDestroyed;
  try {
    const admitted = await admitAiRequirements([
      { kind: "routed", purpose: spec.purpose, target: state.writingTarget.value },
    ]);
    if (!isCurrent()) return;
    if (!admitted) {
      state.phase.value = "closed";
      return;
    }
    markReasoning(instance, spec.source);
    const completed = normalizeGeneratedMarkdown(await spec.executor(job));
    if (!isCurrent()) return;
    if (completed) state.draftText.value = completed;
    state.phase.value = state.draftText.value.trim() ? "review" : "prompt";
  } catch (error) {
    if (!isCurrent()) return;
    const message = getErrorMessage(error);
    state.errorMessage.value = message;
    state.phase.value = "error";
    spec.onError(message);
  } finally {
    if (state.getActiveJob() === job) state.releaseJob(state.phase.value === "closed");
  }
}

export function startEditorTransform(
  job: EditorWritingJob,
  params: {
    action: EditorTransformAction;
    source: TransformSource;
    target: TaskTarget | undefined;
    instruction?: string | undefined;
    previousDraft?: string | undefined;
    documentTitle?: string | undefined;
    references: readonly EditorContextReference[];
    onEvent: (event: TaskStreamEvent) => void;
  },
): Promise<string> {
  const context = extractEditorDocumentContext(job.editor, params.source, {
    documentTitle: params.documentTitle,
    references: params.references,
  });
  return transformEditorTextStream({
    action: params.action,
    text: params.source.text,
    jobId: job.id,
    signal: job.cancellation.signal,
    target: params.target,
    instruction: params.instruction,
    context,
    previousDraft: params.previousDraft,
    onEvent: params.onEvent,
  });
}

export function startEditorTranslation(
  job: EditorWritingJob,
  params: {
    source: TransformSource;
    targetLanguage: TranslationTarget;
    executionTarget: TaskTarget | undefined;
    onEvent: (event: TaskStreamEvent) => void;
  },
): Promise<string> {
  return translateTextStream(
    { text: params.source.text, target: params.targetLanguage, executionTarget: params.executionTarget },
    job.id,
    params.onEvent,
    job.cancellation.signal,
  );
}
