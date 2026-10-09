import { open } from "@tauri-apps/plugin-dialog";
import { computed, onScopeDispose, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { PreparedVoiceReference } from "../contracts/generated/PreparedVoiceReference";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { cancelSpeechGeneration } from "./ipc";
import { authorizeVoiceReferenceSource, prepareVoiceReference } from "./voice-reference-ipc";

export interface VoiceReferenceEditorProps {
  readonly language: string;
  readonly disabled: boolean;
  readonly sourceNode?: SpaceCanvasNode | undefined;
}

export function useVoiceReferenceEditor(
  props: VoiceReferenceEditorProps,
  model: Ref<PreparedVoiceReference | undefined>,
) {
  const { t } = useI18n();
  const sourcePath = computed(() => {
    const data = props.sourceNode?.data;
    return data?.kind === "audio" || data?.kind === "video" ? data.assetPath : undefined;
  });
  const path = shallowRef(sourcePath.value ?? "");
  const start = shallowRef("0");
  const end = shallowRef("0");
  const duration = shallowRef<number>();
  const sourceVersion = shallowRef(0);
  const jobId = shallowRef<string>();
  const error = shallowRef<string>();
  const selectionValid = computed(
    () =>
      Number.isFinite(Number(start.value)) &&
      Number(end.value) - Number(start.value) >= 5 &&
      Number(end.value) - Number(start.value) <= 30 &&
      Number(start.value) >= 0 &&
      duration.value !== undefined &&
      Math.round(Number(end.value) * 1000) <= duration.value,
  );
  const canPrepare = computed(() =>
    Boolean(path.value && selectionValid.value && !jobId.value && !props.disabled),
  );
  let epoch = 0;
  let jobEpoch = 0;
  let disposed = false;
  function isCurrent(version: number) {
    return !disposed && version === epoch;
  }
  async function cancel() {
    const job = jobId.value;
    const version = jobEpoch;
    if (job)
      await cancelSpeechGeneration(job).catch((cause) => {
        if (isCurrent(version) && jobId.value === job) error.value = getErrorMessage(cause);
      });
  }
  function invalidate() {
    epoch += 1;
    model.value = undefined;
    void cancel();
  }
  watch([start, end, () => props.language], invalidate);
  function replaceSource(source: string) {
    invalidate();
    duration.value = undefined;
    start.value = "0";
    end.value = "0";
    path.value = source;
    sourceVersion.value += 1;
  }
  watch([sourcePath, () => props.sourceNode?.data.sourceRevision], ([source]) => {
    if (source) replaceSource(source);
  });
  onScopeDispose(() => {
    disposed = true;
    invalidate();
  });
  async function choose() {
    const version = epoch;
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          { name: t("spaces.speech.reference.title"), extensions: ["wav", "flac", "mp3", "m4a", "ogg"] },
        ],
      });
      if (typeof selected !== "string" || !isCurrent(version)) return;
      const authorizedPath = await authorizeVoiceReferenceSource(selected);
      if (!isCurrent(version)) return;
      replaceSource(authorizedPath);
      error.value = undefined;
    } catch (cause) {
      if (isCurrent(version)) error.value = getErrorMessage(cause);
    }
  }
  async function prepare() {
    if (!canPrepare.value) return;
    const version = ++epoch;
    jobEpoch = version;
    const job = crypto.randomUUID();
    jobId.value = job;
    model.value = undefined;
    error.value = undefined;
    try {
      const prepared = await prepareVoiceReference({
        jobId: job,
        sourcePath: path.value,
        language: props.language,
        startMs: Math.round(Number(start.value) * 1000),
        endMs: Math.round(Number(end.value) * 1000),
      });
      if (props.sourceNode && prepared.sourceRevision !== props.sourceNode.data.sourceRevision)
        throw new Error(t("spaces.speech.reference.sourceChanged"));
      if (isCurrent(version)) {
        model.value = prepared;
        error.value = undefined;
      }
    } catch (cause) {
      if (isCurrent(version) && decodeAppError(cause)?.code !== "cancelled")
        error.value = getErrorMessage(cause);
    } finally {
      if (jobId.value === job) jobId.value = undefined;
    }
  }
  function selectionLoaded(durationMs: number) {
    duration.value = durationMs;
    end.value = String(Math.min(durationMs / 1000, 30));
  }
  return {
    path,
    start,
    end,
    duration,
    sourceVersion,
    jobId,
    error,
    selectionValid,
    canPrepare,
    cancel,
    choose,
    prepare,
    selectionLoaded,
  };
}
