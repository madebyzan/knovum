import { convertFileSrc } from "@tauri-apps/api/core";
import { readonly, shallowRef } from "vue";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { SpeechPreviewAudioChunk } from "../contracts/generated/SpeechPreviewAudioChunk";
import { useSpaceStore } from "../session/space-store";
import type { SpeechAudioOutput, SpeechGenerationProgress } from "./contracts";
import { cancelSpeechGeneration, discardSpeechPreview, previewVoice } from "./ipc";
import { PreviewAudioQueue } from "./preview-audio-queue";

export function useSpeechPreview(setError: (message?: string) => void) {
  const store = useSpaceStore();
  const preview = shallowRef<SpeechAudioOutput & { source: string }>();
  const progress = shallowRef<SpeechGenerationProgress>();
  const jobId = shallowRef<string>();
  const listening = shallowRef(false);
  const audioQueue = new PreviewAudioQueue((playing) => {
    listening.value = playing;
  });

  function discard(path: string) {
    void discardSpeechPreview(path).catch((cause) => setError(getErrorMessage(cause)));
  }

  function clearPreview() {
    audioQueue.stop();
    const path = preview.value?.path;
    preview.value = undefined;
    progress.value = undefined;
    if (path) discard(path);
  }

  function reset() {
    const active = jobId.value;
    if (active) {
      void cancelSpeechGeneration(active).catch((cause) => setError(getErrorMessage(cause)));
    }
    jobId.value = undefined;
    clearPreview();
  }

  function isCurrent(job: string, spaceId: string) {
    return jobId.value === job && store.activeSpaceId === spaceId;
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

  async function previewSelection(
    text: string,
    voiceRevisionId: string,
    pronunciationLexicon: Record<string, string>,
  ) {
    const spaceId = store.activeSpaceId;
    if (!spaceId || jobId.value) return;
    const nextJobId = crypto.randomUUID();
    const previousPath = preview.value?.path;
    jobId.value = nextJobId;
    progress.value = undefined;
    try {
      const started = await audioQueue.start(nextJobId);
      if (!started || jobId.value !== nextJobId || store.activeSpaceId !== spaceId) return;
      const output = await previewVoice(
        { jobId: nextJobId, spaceId, voiceRevisionId, text, pronunciationLexicon },
        (value) => {
          if (jobId.value === nextJobId) progress.value = value;
        },
        (chunk) => handleChunk(chunk, nextJobId, spaceId),
      );
      if (!isCurrent(nextJobId, spaceId)) {
        discard(output.path);
        return;
      }
      preview.value = { ...output, source: convertFileSrc(output.path) };
      if (previousPath) discard(previousPath);
      setError();
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
    if (jobId.value) await cancelSpeechGeneration(jobId.value);
  }

  function stopListening() {
    audioQueue.stop();
  }

  return {
    preview: readonly(preview),
    progress: readonly(progress),
    jobId: readonly(jobId),
    listening: readonly(listening),
    previewSelection,
    clearPreview,
    reset,
    cancel,
    stopListening,
  };
}
