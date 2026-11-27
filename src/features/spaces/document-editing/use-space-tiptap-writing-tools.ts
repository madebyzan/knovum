import type { Editor } from "@tiptap/vue-3";
import { computed, onScopeDispose, readonly, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import {
  LABEL_KEY_BY_TRANSLATION_TARGET,
  TRANSLATION_TARGETS,
  type TranslationTarget,
} from "../../../foundation/ai/translation";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { EditorTransformAction } from "../../../shared/contracts/generated/EditorTransformAction";
import { executeWritingSession, startEditorTransform, startEditorTranslation } from "./tiptap-ai-runner";
import { TONE_ACTION_KEYS } from "./tiptap-writing-catalog";
import {
  createWritingSessionState,
  type LastWritingExecution,
  type RunTransformParams,
} from "./tiptap-writing-session-state";
import {
  acceptGeneratedDraft,
  actionSource,
  insertGeneratedTextBelow,
  type TransformSource,
  type WritingToolsOptions,
} from "./tiptap-writing-text";

export function useSpaceTiptapWritingTools(
  editor: Readonly<Ref<Editor | undefined>>,
  options: WritingToolsOptions = {},
) {
  const { t } = useI18n();
  const state = createWritingSessionState();
  let lastExecution: LastWritingExecution | undefined;

  const tones = computed(() => TONE_ACTION_KEYS.map(([action, key]) => [action, t(key)] as const));
  const translations = computed(() =>
    TRANSLATION_TARGETS.map((target) => [target, t(LABEL_KEY_BY_TRANSLATION_TARGET[target])] as const),
  );
  const onError = (description: string) => toast.error(t("spaces.richtext.ai.failed"), { description });
  const idleEditor = () =>
    !state.getActiveJob() && editor.value && !editor.value.isDestroyed ? editor.value : undefined;

  async function cancel() {
    const job = state.releaseJob(state.phase.value !== "review");
    if (!job) return;
    job.cancellation.abort();
    if (state.phase.value === "streaming")
      state.phase.value = state.draftText.value.trim() ? "review" : "prompt";
    await cancelAiJob(job.id);
  }

  const cancelQuietly = () => {
    if (state.getActiveJob()) void cancel().catch((error: unknown) => toast.error(getErrorMessage(error)));
  };
  function discard() {
    cancelQuietly();
    state.reset(editor.value);
    lastExecution = undefined;
  }
  watch(
    editor,
    (instance, _previous, cleanup) => {
      discard();
      if (!instance) return;
      instance.on("transaction", state.onTransaction);
      cleanup(() => instance.off("transaction", state.onTransaction));
    },
    { flush: "sync" },
  );
  onScopeDispose(cancelQuietly);

  function openPrompt(explicitRange?: { from: number; to: number }) {
    if (editor.value && !editor.value.isDestroyed && editor.value.isEditable)
      state.openPrompt(editor.value, explicitRange);
  }
  async function run(action: EditorTransformAction, params: RunTransformParams = {}) {
    const instance = idleEditor();
    if (!instance) return;
    const source = state.activeSource.value ?? actionSource(instance, action, params.range);
    if (!source)
      return void toast.info(
        t(action === "complete" ? "spaces.richtext.ai.needContext" : "spaces.richtext.ai.selectText"),
      );
    const instruction = params.instruction?.trim() || undefined;
    const previousDraft = params.followUp ? state.draftText.value.trim() || undefined : undefined;
    lastExecution = { kind: "transform", action, instruction };
    await executeWritingSession({
      instance,
      state,
      actionLabel: action,
      purpose: "writing",
      source,
      onError,
      executor: (job) =>
        startEditorTransform(job, {
          action,
          source,
          target: state.writingTarget.value,
          instruction,
          previousDraft,
          documentTitle: options.documentTitle?.value,
          references: state.attachedReferences.value,
          onEvent: (event) => state.onStreamEvent(job, event),
        }),
    });
  }
  async function runTranslation(targetLanguage: TranslationTarget) {
    const instance = idleEditor();
    if (!instance) return;
    const source = state.activeSource.value ?? actionSource(instance, "improve");
    if (!source?.text.trim()) return void toast.info(t("spaces.richtext.ai.selectText"));
    lastExecution = { kind: "translate", action: "improve", translationTarget: targetLanguage };
    await executeWritingSession({
      instance,
      state,
      actionLabel: `translate:${targetLanguage}`,
      purpose: "translation",
      source,
      onError,
      executor: (job) =>
        startEditorTranslation(job, {
          source,
          targetLanguage,
          executionTarget: state.writingTarget.value,
          onEvent: (event) => state.onStreamEvent(job, event),
        }),
    });
  }
  function commitResult(apply: (instance: Editor, source: TransformSource) => boolean) {
    if (!editor.value || !state.activeSource.value || !state.draftText.value.trim()) return;
    if (!apply(editor.value, state.activeSource.value)) toast.error(t("spaces.richtext.ai.applyFailed"));
    else discard();
  }
  return {
    phase: readonly(state.phase),
    runningAction: readonly(state.runningAction),
    writingTarget: state.writingTarget,
    promptText: state.promptText,
    draftText: readonly(state.draftText),
    thinking: readonly(state.thinking),
    thinkingText: readonly(state.thinkingText),
    thinkingDurationMs: readonly(state.thinkingDurationMs),
    errorMessage: readonly(state.errorMessage),
    activeSource: readonly(state.activeSource),
    attachedReferences: readonly(state.attachedReferences),
    tones,
    translations,
    openPrompt,
    attachReference: state.attachReference,
    removeReference: state.removeReference,
    submitPrompt: async () => {
      const instruction = state.promptText.value.trim();
      if (!instruction) return;
      const followUp = state.phase.value === "review" && Boolean(state.draftText.value.trim());
      state.promptText.value = "";
      await run("custom", { instruction, followUp });
    },
    run,
    runTranslation,
    retry: async () => {
      if (!lastExecution) return;
      await (lastExecution.kind === "translate" && lastExecution.translationTarget
        ? runTranslation(lastExecution.translationTarget)
        : run(lastExecution.action, { instruction: lastExecution.instruction }));
    },
    acceptReplace: () =>
      commitResult((instance, source) =>
        acceptGeneratedDraft(instance, {
          action: lastExecution?.action ?? "custom",
          source,
          draft: state.draftText.value,
          promoteTitle: options.promoteTitle,
        }),
      ),
    acceptInsertBelow: () =>
      commitResult((instance, source) => insertGeneratedTextBelow(instance, source, state.draftText.value)),
    cancel,
    discard,
  };
}

export type SpaceTiptapWritingTools = ReturnType<typeof useSpaceTiptapWritingTools>;
