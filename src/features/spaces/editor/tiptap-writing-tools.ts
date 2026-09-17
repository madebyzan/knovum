import type { Editor } from "@tiptap/vue-3";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import {
  transformEditorTextStream,
  type EditorTransformAction,
} from "../../../foundation/ai/editor-transform";
import { translateTextStream, type TranslationTarget } from "../../../foundation/ai/translation";
import { i18n } from "../../../foundation/i18n/i18n";
import { clearReasoningTrace, setReasoningTraceRange } from "./tiptap-reasoning-trace";

type TransformSource = { text: string; from: number; to: number } | { text: string; insertAt: number };
type RunningAction = EditorTransformAction | `translate:${TranslationTarget}`;

const itemClass =
  "flex h-9 w-full cursor-default items-center gap-2.5 rounded-control px-2.5 text-left text-label text-foreground outline-none transition-colors hover:bg-control-hover data-[highlighted]:bg-control-hover data-[disabled]:pointer-events-none data-[disabled]:opacity-40";
const submenuClass =
  "z-50 w-56 overflow-hidden rounded-dialog border border-border bg-overlay p-1.5 shadow-popover outline-none";

export function useSpaceTiptapWritingTools(editor: Editor) {
  const { t } = useI18n();
  const runningAction = ref<RunningAction>();
  const activeJobId = ref<string>();
  const tones = computed(() => toneOptions());
  const translations = computed(() => translationOptions());

  async function cancel() {
    if (activeJobId.value) {
      const jobId = activeJobId.value;
      activeJobId.value = undefined;
      await cancelAiJob(jobId);
    }
  }

  async function executeTransformAction(
    actionLabel: RunningAction,
    source: TransformSource | undefined,
    actionContext: EditorTransformAction,
    transform: (text: string, jobId: string) => Promise<string>,
  ) {
    if (runningAction.value) return;
    if (!source) return notifyMissingSource(actionContext);
    const jobId = crypto.randomUUID();
    runningAction.value = actionLabel;
    activeJobId.value = jobId;
    markReasoning(editor, source);
    try {
      const generated = await transform(source.text, jobId);
      if (!applyGeneratedText(editor, actionContext, source, generated))
        toast.error(t("editor.aiApplyFailed"));
    } catch (error) {
      toast.error(t("editor.aiFailed"), { description: String(error) });
    } finally {
      clearReasoningTrace(editor.view);
      runningAction.value = undefined;
      activeJobId.value = undefined;
    }
  }

  async function run(action: EditorTransformAction) {
    await executeTransformAction(action, actionSource(editor, action), action, (text, jobId) =>
      transformEditorTextStream(action, text, jobId, () => {}),
    );
  }

  async function runTranslation(target: TranslationTarget) {
    await executeTransformAction(`translate:${target}`, selectionSource(editor), "improve", (text, jobId) =>
      translateTextStream({ text, target }, jobId, () => {}),
    );
  }

  return { runningAction, cancel, itemClass, submenuClass, tones, translations, run, runTranslation };
}

function markReasoning(editor: Editor, source: TransformSource) {
  if (!("from" in source)) return;
  setReasoningTraceRange(editor.view, source.from, source.to);
}

function notifyMissingSource(action: EditorTransformAction) {
  toast.info(i18n.global.t(action === "complete" ? "editor.aiNeedContext" : "editor.aiSelectText"));
}

function actionSource(editor: Editor, action: EditorTransformAction): TransformSource | undefined {
  return action === "complete" ? completionSource(editor) : selectionSource(editor);
}

function selectionSource(editor: Editor): TransformSource | undefined {
  const { from, to } = editor.state.selection;
  return from === to ? undefined : { from, to, text: editor.state.doc.textBetween(from, to, "\n", "\n") };
}

function completionSource(editor: Editor): TransformSource | undefined {
  const { from, to, $from } = editor.state.selection;
  if (from !== to) return { insertAt: to, text: editor.state.doc.textBetween(from, to, "\n", "\n") };
  const blockStart = $from.start($from.depth);
  const text = editor.state.doc.textBetween(blockStart, from, "\n", "\n");
  return text.trim() ? { insertAt: from, text } : undefined;
}

function applyGeneratedText(
  editor: Editor,
  action: EditorTransformAction,
  source: TransformSource,
  generated: string,
) {
  if (action === "complete" && "insertAt" in source) {
    return editor
      .chain()
      .focus()
      .setTextSelection(source.insertAt)
      .insertContent(generated, { contentType: "markdown" })
      .run();
  }
  if (!("from" in source)) return false;
  return editor
    .chain()
    .focus()
    .insertContentAt({ from: source.from, to: source.to }, generated, { contentType: "markdown" })
    .run();
}

function toneOptions() {
  const t = i18n.global.t;
  return [
    ["tone_academic", t("editor.aiTone.academic")],
    ["tone_business", t("editor.aiTone.business")],
    ["tone_casual", t("editor.aiTone.casual")],
    ["tone_confident", t("editor.aiTone.confident")],
    ["tone_creative", t("editor.aiTone.creative")],
    ["tone_friendly", t("editor.aiTone.friendly")],
    ["tone_humorous", t("editor.aiTone.humorous")],
    ["tone_informative", t("editor.aiTone.informative")],
    ["tone_inspirational", t("editor.aiTone.inspirational")],
    ["tone_objective", t("editor.aiTone.objective")],
    ["tone_persuasive", t("editor.aiTone.persuasive")],
  ] as const satisfies readonly (readonly [EditorTransformAction, string])[];
}

function translationOptions() {
  const t = i18n.global.t;
  return [
    ["zh-CN", t("editor.aiLanguage.zhCn")],
    ["zh-TW", t("editor.aiLanguage.zhTw")],
    ["en", t("editor.aiLanguage.en")],
    ["ja", t("editor.aiLanguage.ja")],
    ["ko", t("editor.aiLanguage.ko")],
    ["fr", t("editor.aiLanguage.fr")],
    ["de", t("editor.aiLanguage.de")],
    ["es", t("editor.aiLanguage.es")],
    ["pt", t("editor.aiLanguage.pt")],
    ["it", t("editor.aiLanguage.it")],
    ["ru", t("editor.aiLanguage.ru")],
    ["ar", t("editor.aiLanguage.ar")],
  ] as const satisfies readonly (readonly [TranslationTarget, string])[];
}
