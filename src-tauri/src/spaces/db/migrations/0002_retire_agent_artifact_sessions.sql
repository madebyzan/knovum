-- Create with Agent was removed, and these sessions were its only state. Nothing can drive one of
-- its Runs any more, so a Run it left open settles as failed, as startup recovery settled an
-- interrupted session; the Run and its events stay in the ledger. The rows hold run state, not
-- content: what an Agent produced was imported into the Space or is gone with its workspace.
UPDATE spaces_runs
SET state = 'failed',
    state_version = state_version + 1,
    updated_at = CAST((julianday('now') - 2440587.5) * 86400000 AS INTEGER),
    completed_at = CAST((julianday('now') - 2440587.5) * 86400000 AS INTEGER)
WHERE completed_at IS NULL
  AND id IN (SELECT run_id FROM spaces_agent_artifact_sessions);

DROP TABLE spaces_agent_artifact_sessions;
