import { convertFileSrc } from "@tauri-apps/api/core";
import { readonly, shallowRef } from "vue";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
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

function buildPreviewRequest(job: { id: string; spaceId: string }, input: PreviewInput) {
  const instruction = input.deliveryInstruction.trim();
  return {
    jobId: job.id,
    spaceId: job.spaceId,
    source: input.source,
    text: input.text,
    pronunciationLexicon: input.pronunciationLexicon,
    ...(instruction ? { deliveryInstruction: instruction } : {}),
  };
}

export function useSpeechPreview(
  setError: (message?: string) => void,
  ownerSpaceId: () => string | undefined,
) {
  const preview = shallowRef<SpeechAudioOutput & { source: string }>();
  const progress = shallowRef<SpeechGenerationProgress>();
  const jobId = shallowRef<string>();
  const listening = shallowRef(false);
  const audioQueue = new PreviewAudioQueue((playing) => {
    listening.value = playing;
  });

  function discard(path: string, reportFailure = true) {
    void discardSpeechPreview(path).catch((cause) => {
      if (reportFailure) setError(getErrorMessage(cause));
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
    const active = jobId.value;
    if (active) void cancelSpeechGeneration(active).catch(() => undefined);
    jobId.value = undefined;
    clearPreview(false);
  }

  function isCurrent(job: string, spaceId: string) {
    return jobId.value === job && ownerSpaceId() === spaceId;
  }

  function clearJob(job: string) {
    if (jobId.value === job) jobId.value = undefined;
  }

  function handleChunk(chunk: SpeechPreviewAudioChunk, job: string, spaceId: string) {
    if (!isCurrent(job, spaceId)) return;
    try {
      audioQueue.enqueue(chunk);
    } catch (cause) {
      jobId.value = undefined;
      audioQueue.stop(job);
      void cancelSpeechGeneration(job).catch((cancelCause) => setError(getErrorMessage(cancelCause)));
      setError(getErrorMessage(cause));
    }
  }

  function updateProgress(value: SpeechGenerationProgress, job: string) {
    if (jobId.value === job) progress.value = value;
  }

  function commitPreviewOutput(
    output: SpeechAudioOutput,
    expectedJobId: string,
    spaceId: string,
    previousPath?: string,
  ) {
    if (!isCurrent(expectedJobId, spaceId)) {
      discard(output.path);
      return;
    }
    preview.value = { ...output, source: convertFileSrc(output.path) };
    if (previousPath) discard(previousPath);
    setError();
  }

  async function previewSource(
    text: string,
    source: VoicePreviewSource,
    pronunciationLexicon: Record<string, string>,
    deliveryInstruction = "",
  ) {
    const spaceId = ownerSpaceId();
    if (!spaceId || jobId.value) return;
    const nextJobId = crypto.randomUUID();
    const previousPath = preview.value?.path;
    jobId.value = nextJobId;
    progress.value = undefined;
    try {
      const started = await audioQueue.start(nextJobId);
      if (!started || jobId.value !== nextJobId || ownerSpaceId() !== spaceId) return;
      const output = await previewVoice(
        buildPreviewRequest(
          { id: nextJobId, spaceId },
          { text, source, pronunciationLexicon, deliveryInstruction },
        ),
        (value) => updateProgress(value, nextJobId),
        (chunk) => handleChunk(chunk, nextJobId, spaceId),
      );
      commitPreviewOutput(output, nextJobId, spaceId, previousPath);
    } catch (cause) {
      audioQueue.stop(nextJobId);
      reportPreviewFailure(cause, nextJobId);
    } finally {
      clearJob(nextJobId);
    }
  }

  function reportPreviewFailure(cause: unknown, expectedJobId: string) {
    if (jobId.value === expectedJobId && decodeAppError(cause)?.code !== "cancelled") {
      setError(getErrorMessage(cause));
    }
  }

  async function cancel() {
    audioQueue.stop();
    const active = jobId.value;
    if (active) await cancelSpeechGeneration(active).catch((cause) => reportPreviewFailure(cause, active));
  }

  function stopListening() {
    audioQueue.stop();
  }

  return {
    preview: readonly(preview),
    progress: readonly(progress),
    jobId: readonly(jobId),
    listening: readonly(listening),
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
