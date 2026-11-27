import type { Editor } from "@tiptap/vue-3";
import { shallowRef } from "vue";

import type { TranslationTarget } from "../../../foundation/ai/translation";
import type { EditorContextReference } from "../../../shared/contracts/generated/EditorContextReference";
import type { EditorTransformAction } from "../../../shared/contracts/generated/EditorTransformAction";
import type { TaskStreamEvent } from "../../../shared/contracts/generated/TaskStreamEvent";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import { clearReasoningTrace } from "./tiptap-reasoning-trace";
import type { RunningAction } from "./tiptap-writing-catalog";
import { captureSelectionOrCursor, markReasoning, type TransformSource } from "./tiptap-writing-text";

export interface LastWritingExecution {
  readonly kind: "transform" | "translate";
  readonly action: EditorTransformAction;
  readonly translationTarget?: TranslationTarget;
  readonly instruction?: string | undefined;
}

export interface RunTransformParams {
  readonly instruction?: string | undefined;
  readonly range?: { from: number; to: number } | undefined;
  readonly followUp?: boolean | undefined;
}

export interface EditorWritingJob {
  readonly id: string;
  readonly editor: Editor;
  readonly cancellation: AbortController;
}

export type AiSessionPhase = "closed" | "prompt" | "streaming" | "review" | "error";

export function createWritingSessionState() {
  const phase = shallowRef<AiSessionPhase>("closed");
  const runningAction = shallowRef<RunningAction>();
  const writingTarget = shallowRef<TaskTarget>();
  const promptText = shallowRef("");
  const draftText = shallowRef("");
  const thinking = shallowRef(false);
  const thinkingText = shallowRef("");
  const thinkingDurationMs = shallowRef<number | undefined>(undefined);
  const errorMessage = shallowRef<string>();
  const activeSource = shallowRef<TransformSource>();
  const attachedReferences = shallowRef<readonly EditorContextReference[]>([]);
  let activeJob: EditorWritingJob | undefined;
  let thinkingStartedAt: number | undefined;

  function clearThinking() {
    thinking.value = false;
    thinkingText.value = "";
    thinkingDurationMs.value = undefined;
    thinkingStartedAt = undefined;
  }

  function beginJob(instance: Editor, actionLabel: RunningAction, source: TransformSource): EditorWritingJob {
    const job: EditorWritingJob = {
      id: crypto.randomUUID(),
      editor: instance,
      cancellation: new AbortController(),
    };
    activeJob = job;
    activeSource.value = source;
    runningAction.value = actionLabel;
    draftText.value = "";
    errorMessage.value = undefined;
    clearThinking();
    phase.value = "streaming";
    return job;
  }

  function releaseJob(clearTrace = false): EditorWritingJob | undefined {
    const job = activeJob;
    activeJob = undefined;
    runningAction.value = undefined;
    if (thinkingStartedAt !== undefined && thinkingDurationMs.value === undefined)
      thinkingDurationMs.value = Math.max(0, Date.now() - thinkingStartedAt);
    thinking.value = false;
    if (clearTrace && job && !job.editor.isDestroyed) clearReasoningTrace(job.editor.view);
    return job;
  }

  function openPrompt(instance: Editor, explicitRange?: { from: number; to: number }) {
    const captured = captureSelectionOrCursor(instance, explicitRange);
    activeSource.value = captured;
    markReasoning(instance, captured);
    draftText.value = "";
    errorMessage.value = undefined;
    promptText.value = "";
    clearThinking();
    phase.value = "prompt";
  }

  function reset(instance?: Editor) {
    if (instance && !instance.isDestroyed) clearReasoningTrace(instance.view);
    phase.value = "closed";
    promptText.value = "";
    draftText.value = "";
    errorMessage.value = undefined;
    activeSource.value = undefined;
    clearThinking();
  }

  function freezeThinkingDuration() {
    if (thinkingStartedAt !== undefined && thinkingDurationMs.value === undefined)
      thinkingDurationMs.value = Math.max(0, Date.now() - thinkingStartedAt);
  }

  function appendThinking(text: string) {
    thinking.value = true;
    thinkingStartedAt ??= Date.now();
    if (text) thinkingText.value += text;
  }

  function onStreamEvent(job: EditorWritingJob, event: TaskStreamEvent) {
    if (activeJob !== job) return;
    if (event.event === "thinking_delta") appendThinking(event.text);
    else if (event.event === "delta") {
      freezeThinkingDuration();
      thinking.value = false;
      draftText.value += event.text;
    }
  }

  function attachReference(reference: EditorContextReference) {
    if (attachedReferences.value.some((item) => item.nodeId === reference.nodeId)) return;
    attachedReferences.value = [...attachedReferences.value.slice(0, 4), reference];
  }

  function removeReference(nodeId: string) {
    attachedReferences.value = attachedReferences.value.filter((item) => item.nodeId !== nodeId);
  }

  function onTransaction({
    transaction,
  }: {
    readonly transaction: {
      readonly docChanged: boolean;
      readonly mapping: { map: (pos: number, assoc?: number) => number };
    };
  }) {
    const source = activeSource.value;
    if (!transaction.docChanged || !source || phase.value === "closed") return;
    const from = transaction.mapping.map(source.from, 1);
    const to = Math.max(from, transaction.mapping.map(source.to, -1));
    if (from !== source.from || to !== source.to) activeSource.value = { ...source, from, to };
  }

  return {
    phase,
    runningAction,
    writingTarget,
    promptText,
    draftText,
    thinking,
    thinkingText,
    thinkingDurationMs,
    errorMessage,
    activeSource,
    attachedReferences,
    getActiveJob: () => activeJob,
    beginJob,
    releaseJob,
    openPrompt,
    reset,
    onStreamEvent,
    attachReference,
    removeReference,
    onTransaction,
  };
}

export type WritingSessionState = ReturnType<typeof createWritingSessionState>;
