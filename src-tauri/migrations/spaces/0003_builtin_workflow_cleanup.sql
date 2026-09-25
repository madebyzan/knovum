-- Built-in recipes now persist their definition directly in the Run snapshot.
-- Pre-release Workflow Studio drafts, published copies, and automation rules have no current executor.
DROP TABLE spaces_automation_occurrences;
DROP TABLE spaces_automation_events;
DROP TABLE spaces_automation_triggers;
DROP TABLE spaces_workflow_drafts;
DROP TABLE spaces_workflow_versions;
DROP TABLE spaces_run_batch_items;
ALTER TABLE spaces_authorization_grants DROP COLUMN egress_targets;

UPDATE spaces_run_attempts SET state='failed',finished_at=unixepoch('now')*1000,
  error_code='workflow-executor-removed'
WHERE run_id IN (SELECT id FROM spaces_runs WHERE kind='automation' AND completed_at IS NULL)
  AND state='executing';
UPDATE spaces_run_steps SET state='blocked',wait_reason=NULL,state_version=state_version+1,
  updated_at=unixepoch('now')*1000
WHERE run_id IN (SELECT id FROM spaces_runs WHERE kind='automation' AND completed_at IS NULL)
  AND state IN ('pending','ready','running','waiting','paused');
UPDATE spaces_runs SET state='failed',wait_reason=NULL,state_version=state_version+1,
  completed_at=unixepoch('now')*1000,updated_at=unixepoch('now')*1000
WHERE kind='automation' AND completed_at IS NULL;
DELETE FROM spaces_authorization_grants WHERE principal_kind='automation';

DELETE FROM spaces_workflow_templates
WHERE json_extract(definition,'$.schemaVersion') IS NOT 3;
ALTER TABLE spaces_workflow_templates DROP COLUMN dependencies;
ALTER TABLE spaces_workflow_templates DROP COLUMN source_workflow_id;
ALTER TABLE spaces_workflow_templates DROP COLUMN source_workflow_version;
