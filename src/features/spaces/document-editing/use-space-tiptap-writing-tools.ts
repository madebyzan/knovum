import type { Editor } from "@tiptap/vue-3";
import { computed, onScopeDispose, readonly, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { admitAiRequirements } from "../../../foundation/ai/ai-readiness-admission";
import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import { transformEditorTextStream } from "../../../foundation/ai/editor-transform";
import {
  LABEL_KEY_BY_TRANSLATION_TARGET,
  translateTextStream,
  TRANSLATION_TARGETS,
  type TranslationTarget,
} from "../../../foundation/ai/translation";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { EditorTransformAction } from "../../../shared/contracts/generated/EditorTransformAction";
import type { RoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";
import { clearReasoningTrace } from "./tiptap-reasoning-trace";
import { actionSource, applyGeneratedText, markReasoning } from "./tiptap-writing-text";

type RunningAction = EditorTransformAction | `translate:${TranslationTarget}`;
interface EditorWritingJob {
  id: string;
  editor: Editor;
  cancellation: AbortController;
}

export function useSpaceTiptapWritingTools(editor: Readonly<Ref<Editor | undefined>>) {
  const { t } = useI18n();
  const runningAction = shallowRef<RunningAction>();
  let activeJob: EditorWritingJob | undefined;
  const tones = computed(
    () =>
      [
        ["tone_academic", t("spaces.richtext.ai.tones.academic")],
        ["tone_business", t("spaces.richtext.ai.tones.business")],
        ["tone_casual", t("spaces.richtext.ai.tones.casual")],
        ["tone_confident", t("spaces.richtext.ai.tones.confident")],
        ["tone_creative", t("spaces.richtext.ai.tones.creative")],
        ["tone_friendly", t("spaces.richtext.ai.tones.friendly")],
        ["tone_humorous", t("spaces.richtext.ai.tones.humorous")],
        ["tone_informative", t("spaces.richtext.ai.tones.informative")],
        ["tone_inspirational", t("spaces.richtext.ai.tones.inspirational")],
        ["tone_objective", t("spaces.richtext.ai.tones.objective")],
        ["tone_persuasive", t("spaces.richtext.ai.tones.persuasive")],
      ] as const satisfies readonly (readonly [EditorTransformAction, string])[],
  );
  const translations = computed(() =>
    TRANSLATION_TARGETS.map((target) => [target, t(LABEL_KEY_BY_TRANSLATION_TARGET[target])] as const),
  );

  function notifyMissingSource(action: EditorTransformAction) {
    toast.info(t(action === "complete" ? "spaces.richtext.ai.needContext" : "spaces.richtext.ai.selectText"));
  }

  function releaseJob() {
    const job = activeJob;
    activeJob = undefined;
    runningAction.value = undefined;
    if (job && !job.editor.isDestroyed) clearReasoningTrace(job.editor.view);
    return job;
  }
  async function cancel() {
    const job = releaseJob();
    if (!job) return;
    job.cancellation.abort();
    await cancelAiJob(job.id);
  }

  function cancelInBackground() {
    void cancel().catch((error: unknown) => toast.error(getErrorMessage(error)));
  }
  watch(editor, cancelInBackground, { flush: "sync" });
  onScopeDispose(cancelInBackground);

  /** Claims the editor for one writing task on what `action` reads, or explains why there is nothing. */
  function claimJob(actionLabel: RunningAction, action: EditorTransformAction) {
    const instance = editor.value;
    if (activeJob || !instance || instance.isDestroyed) return undefined;
    const source = actionSource(instance, action);
    if (!source) return notifyMissingSource(action);
    const job: EditorWritingJob = {
      id: crypto.randomUUID(),
      editor: instance,
      cancellation: new AbortController(),
    };
    runningAction.value = actionLabel;
    activeJob = job;
    return { job, source };
  }

  async function executeTransformAction(
    actionLabel: RunningAction,
    actionContext: EditorTransformAction,
    purpose: Extract<RoutingPurpose, "writing" | "translation">,
    transform: (text: string, jobId: string, signal: AbortSignal) => Promise<string>,
  ) {
    const claimed = claimJob(actionLabel, actionContext);
    if (!claimed) return;
    const { job, source } = claimed;
    const instance = job.editor;
    const isCurrent = () => activeJob === job && !instance.isDestroyed;
    try {
      // The task is claimed before admission, so the editor stays busy while it is decided.
      if (!(await admitAiRequirements([{ kind: "routed", purpose }])) || !isCurrent()) return;
      markReasoning(instance, source);
      const generated = await transform(source.text, job.id, job.cancellation.signal);
      if (isCurrent()) applyResult(instance, actionContext, source, generated);
    } catch (error) {
      if (isCurrent()) toast.error(t("spaces.richtext.ai.failed"), { description: getErrorMessage(error) });
    } finally {
      if (activeJob === job) releaseJob();
    }
  }

  function applyResult(...result: Parameters<typeof applyGeneratedText>) {
    if (!applyGeneratedText(...result)) toast.error(t("spaces.richtext.ai.applyFailed"));
  }

  async function run(action: EditorTransformAction) {
    await executeTransformAction(action, action, "writing", (text, jobId, signal) =>
      transformEditorTextStream({ action, text, jobId, signal, onEvent: () => {} }),
    );
  }

  async function runTranslation(target: TranslationTarget) {
    await executeTransformAction(`translate:${target}`, "improve", "translation", (text, jobId, signal) =>
      translateTextStream({ text, target }, jobId, () => {}, signal),
    );
  }

  return { runningAction: readonly(runningAction), cancel, tones, translations, run, runTranslation };
}

export type SpaceTiptapWritingTools = ReturnType<typeof useSpaceTiptapWritingTools>;
