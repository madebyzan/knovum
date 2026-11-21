-- Automatic background visual understanding follows the user's current visual routing in Settings
-- instead of staying pinned to an earlier default executor that may no longer work. Clear stored
-- targets on unfinished visual jobs so queued, waiting, and failed-retryable jobs immediately pick
-- up the active Settings route; explicit per-node requests still store their chosen target.
UPDATE spaces_enrichment_jobs
SET execution_target = NULL
WHERE job_type = 'analyze_visual'
  AND status IN ('pending', 'running', 'failed', 'awaiting_configuration')
  AND execution_target IS NOT NULL;
