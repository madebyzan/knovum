-- A library that ran a build carrying 0005 but not 0006 still held the first reader's
-- textRecognition records, which that build's validation refuses. Every result written to such a
-- node was rejected, so the document parses 0005 queued failed and left their documents without
-- coverage, and other work on those nodes failed the same way. 0006 withdrew the records but cannot
-- revive a job that already failed, and a failed job counts as answered, so nothing reads those
-- nodes again. Only that refusal is retried: the node could not change after it, so the job's
-- revision is still the node's, and any other failure keeps its outcome and its Retry action.
-- One job per node, type and revision may be live, so only the latest refused one is revived.
UPDATE spaces_enrichment_jobs
  SET status = 'pending', error = NULL, updated_at = CAST(strftime('%s', 'now') AS INTEGER) * 1000
  WHERE id IN (
    SELECT (
      SELECT latest.id FROM spaces_enrichment_jobs latest
      WHERE latest.space_id = j.space_id AND latest.node_id = j.node_id
        AND latest.job_type = j.job_type AND latest.source_revision = j.source_revision
        AND latest.status = 'failed'
        AND latest.error LIKE '%textRecognition field ''text'' is not supported%'
      ORDER BY latest.updated_at DESC, latest.id DESC LIMIT 1
    )
    FROM spaces_enrichment_jobs j
    JOIN spaces_nodes n ON n.space_id = j.space_id AND n.id = j.node_id AND n.deleted_at IS NULL
    WHERE j.status = 'failed'
      AND j.error LIKE '%textRecognition field ''text'' is not supported%'
      AND NOT EXISTS (
        SELECT 1 FROM spaces_enrichment_jobs live
        WHERE live.space_id = j.space_id AND live.node_id = j.node_id
          AND live.job_type = j.job_type AND live.source_revision = j.source_revision
          AND live.status IN ('pending', 'running', 'awaiting_configuration')
      )
  );
