-- Add 'analyze_media' job type introduced by the audio asset lifecycle refactor.
-- SQLite does not support ALTER COLUMN, so the table is rebuilt with the expanded CHECK.
CREATE TABLE spaces_enrichment_jobs_next (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  job_type TEXT NOT NULL CHECK (job_type IN ('embed_text','analyze_image','analyze_media','transcribe_media','analyze_video_first_frame','embed_media_transcript','parse_document','analyze_document')),
  status TEXT NOT NULL CHECK (status IN ('pending','running','succeeded','failed','cancelled','awaiting_configuration')),
  source_revision TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
  error TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE
);
INSERT INTO spaces_enrichment_jobs_next(id,space_id,node_id,job_type,status,source_revision,attempts,error,created_at,updated_at)
SELECT id,space_id,node_id,job_type,status,source_revision,attempts,error,created_at,updated_at
FROM spaces_enrichment_jobs;
DROP TABLE spaces_enrichment_jobs;
ALTER TABLE spaces_enrichment_jobs_next RENAME TO spaces_enrichment_jobs;
CREATE UNIQUE INDEX spaces_enrichment_jobs_unique_pending
  ON spaces_enrichment_jobs(space_id,node_id,job_type,source_revision)
  WHERE status IN ('pending','running','awaiting_configuration');
CREATE INDEX spaces_enrichment_jobs_status ON spaces_enrichment_jobs(status,updated_at);
