import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef, type Ref } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { EnrichmentJobState } from "../contracts/generated/EnrichmentJobState";
import type { EnrichmentJobType } from "../contracts/generated/EnrichmentJobType";
import { retryEnrichmentJob } from "./enrichment-jobs";
import { enrichmentJobsQuery } from "./enrichment-jobs-query";

export const MEDIA_ENRICHMENT_JOB_TYPES: readonly EnrichmentJobType[] = [
  "analyze_media",
  "transcribe_media",
  "align_media_transcript",
  "diarize_media_transcript",
];

export interface NodeEnrichmentStatus {
  pending: number;
  running: number;
  failed: number;
  waiting: number;
  transcribing?: boolean;
  lastError?: string;
}

export function useNodeEnrichmentStatus(
  spaceId: Ref<string | undefined>,
  nodeId: Ref<string | undefined>,
  jobTypes?: readonly EnrichmentJobType[],
) {
  const retrying = shallowRef(false);
  const allowedTypes = jobTypes ? new Set<EnrichmentJobType>(jobTypes) : undefined;
  const query = useQuery(computed(() => enrichmentJobsQuery(Boolean(spaceId.value && nodeId.value))));
  const jobs = computed<EnrichmentJobState[]>(() => {
    if (!spaceId.value || !nodeId.value) return [];
    return (query.data.value ?? [])
      .map(({ job }) => job)
      .filter(
        (job) =>
          job.spaceId === spaceId.value &&
          job.nodeId === nodeId.value &&
          (!allowedTypes || allowedTypes.has(job.jobType)),
      );
  });
  const status = computed<NodeEnrichmentStatus>(() => summarizeNodeEnrichment(jobs.value));
  async function refresh() {
    if (spaceId.value && nodeId.value) await query.refetch();
  }

  async function retryFailed() {
    const failed = jobs.value.filter((job) => job.status === "failed");
    if (!failed.length || !spaceId.value || retrying.value) return;
    retrying.value = true;
    try {
      for (const job of failed) {
        // oxlint-disable-next-line eslint/no-await-in-loop -- Retries share the node's projection stream.
        await retryEnrichmentJob(job.id);
      }
    } catch (error) {
      toast.error(getErrorMessage(error));
    } finally {
      retrying.value = false;
      await refresh();
    }
  }

  return { status, jobs, loading: query.isPending, retrying, refresh, retryFailed };
}

function summarizeNodeEnrichment(jobs: readonly EnrichmentJobState[]): NodeEnrichmentStatus {
  let pending = 0;
  let running = 0;
  let failed = 0;
  let waiting = 0;
  let lastError: string | undefined;
  for (const job of jobs) {
    if (job.status === "pending") pending += 1;
    else if (job.status === "running") running += 1;
    else if (job.status === "awaiting_configuration") waiting += 1;
    else if (job.status === "failed") {
      failed += 1;
      lastError = job.error ?? lastError;
    }
  }
  const transcribing = jobs.some(isActiveTranscriptionJob);
  return lastError === undefined
    ? { pending, running, failed, waiting, transcribing }
    : { pending, running, failed, waiting, transcribing, lastError };
}

function isActiveTranscriptionJob(job: EnrichmentJobState) {
  return (
    (job.jobType === "analyze_media" || job.jobType === "transcribe_media") &&
    (job.status === "pending" || job.status === "running")
  );
}
