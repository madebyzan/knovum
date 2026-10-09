import { convertFileSrc } from "@tauri-apps/api/core";
import { error as logError } from "@tauri-apps/plugin-log";
import { readonly, shallowRef } from "vue";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { formatErrorDiagnostic, getErrorMessage } from "../../../foundation/errors/error-format";
import type { SpeechAudioOutput } from "../contracts/generated/SpeechAudioOutput";
import type { SpeechGenerationProgress } from "../contracts/generated/SpeechGenerationProgress";
import type { SpeechPreviewAudioChunk } from "../contracts/generated/SpeechPreviewAudioChunk";
import type { VoicePreviewSource } from "../contracts/generated/VoicePreviewSource";
import { cancelSpeechGeneration, discardSpeechPreview, previewVoice } from "./ipc";
import { PreviewAudioQueue } from "./preview-audio-queue";

interface PreviewInput {
  text: string;
  source: VoicePreviewSource;
  pronunciationLexicon: Record<string, string>;
  deliveryInstruction: string;
}

function buildPreviewRequest(jobId: string, input: PreviewInput) {
  const instruction = input.deliveryInstruction.trim();
  return {
    jobId,
    source: input.source,
    text: input.text,
    pronunciationLexicon: input.pronunciationLexicon,
    ...(instruction ? { deliveryInstruction: instruction } : {}),
  };
}

export function useSpeechPreview() {
  const preview = shallowRef<SpeechAudioOutput & { source: string }>();
  const progress = shallowRef<SpeechGenerationProgress>();
  const jobId = shallowRef<string>();
  const listening = shallowRef(false);
  const error = shallowRef<string>();
  let errorEpoch = 0;
  const audioQueue = new PreviewAudioQueue((playing) => {
    listening.value = playing;
  });

  function discard(path: string, reportFailure = true, expectedEpoch = errorEpoch) {
    void discardSpeechPreview(path).catch((cause) => {
      if (reportFailure && expectedEpoch === errorEpoch && !error.value) {
        error.value = getErrorMessage(cause);
      } else {
        void logError(`Speech preview cleanup failed: ${formatErrorDiagnostic(cause)}`);
      }
    });
  }

  function clearPreview(reportCleanupFailure = true) {
    audioQueue.stop();
    const path = preview.value?.path;
    preview.value = undefined;
    progress.value = undefined;
    if (path) discard(path, reportCleanupFailure);
  }

  function reset() {
    errorEpoch += 1;
    const active = jobId.value;
    if (active)
      void cancelSpeechGeneration(active).catch((cause) => {
        void logError(`Speech preview reset cancellation failed: ${formatErrorDiagnostic(cause)}`);
      });
    jobId.value = undefined;
    error.value = undefined;
    clearPreview(false);
  }

  function clearJob(job: string) {
    if (jobId.value === job) jobId.value = undefined;
  }

  function handleChunk(chunk: SpeechPreviewAudioChunk, job: string) {
    if (jobId.value !== job) return;
    try {
      audioQueue.enqueue(chunk);
    } catch (cause) {
      const expectedEpoch = errorEpoch;
      jobId.value = undefined;
      audioQueue.stop(job);
      error.value = getErrorMessage(cause);
      void cancelSpeechGeneration(job).catch((cancelCause) => {
        if (expectedEpoch === errorEpoch && !error.value) error.value = getErrorMessage(cancelCause);
        else
          void logError(`Speech preview failure cancellation failed: ${formatErrorDiagnostic(cancelCause)}`);
      });
    }
  }

  function updateProgress(value: SpeechGenerationProgress, job: string) {
    if (jobId.value === job) progress.value = value;
  }

  function commitPreviewOutput(output: SpeechAudioOutput, expectedJobId: string, previousPath?: string) {
    if (jobId.value !== expectedJobId) {
      discard(output.path, false);
      return;
    }
    preview.value = { ...output, source: convertFileSrc(output.path) };
    if (previousPath) discard(previousPath);
    error.value = undefined;
  }

  async function previewSource(
    text: string,
    source: VoicePreviewSource,
    pronunciationLexicon: Record<string, string>,
    deliveryInstruction = "",
  ) {
    if (jobId.value) return;
    errorEpoch += 1;
    const nextJobId = crypto.randomUUID();
    const previousPath = preview.value?.path;
    jobId.value = nextJobId;
    progress.value = undefined;
    error.value = undefined;
    try {
      const started = await audioQueue.start(nextJobId);
      if (!started || jobId.value !== nextJobId) return;
      const output = await previewVoice(
        buildPreviewRequest(nextJobId, { text, source, pronunciationLexicon, deliveryInstruction }),
        (value) => updateProgress(value, nextJobId),
        (chunk) => handleChunk(chunk, nextJobId),
      );
      commitPreviewOutput(output, nextJobId, previousPath);
    } catch (cause) {
      audioQueue.stop(nextJobId);
      reportPreviewFailure(cause, nextJobId);
    } finally {
      clearJob(nextJobId);
    }
  }

  function reportPreviewFailure(cause: unknown, expectedJobId: string) {
    if (jobId.value === expectedJobId && decodeAppError(cause)?.code !== "cancelled") {
      error.value = getErrorMessage(cause);
    }
  }

  async function cancel() {
    audioQueue.stop();
    const active = jobId.value;
    const expectedEpoch = errorEpoch;
    if (active)
      await cancelSpeechGeneration(active).catch((cause) => {
        if (jobId.value === active && expectedEpoch === errorEpoch && !error.value) {
          error.value = getErrorMessage(cause);
        } else {
          void logError(`Speech preview cancellation failed: ${formatErrorDiagnostic(cause)}`);
        }
      });
  }

  function stopListening() {
    audioQueue.stop();
  }

  return {
    preview: readonly(preview),
    progress: readonly(progress),
    jobId: readonly(jobId),
    listening: readonly(listening),
    error: readonly(error),
    previewSource,
    previewSelection: (
      text: string,
      revisionId: string,
      lexicon: Record<string, string>,
      deliveryInstruction = "",
    ) => previewSource(text, { kind: "revision", revisionId }, lexicon, deliveryInstruction),
    clearPreview,
    reset,
    cancel,
    stopListening,
  };
}
