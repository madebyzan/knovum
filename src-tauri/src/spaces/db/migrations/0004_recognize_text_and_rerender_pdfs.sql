-- Local text recognition is a durable enrichment job, which the job_type CHECK must admit. SQLite
-- cannot alter a CHECK, so the table is rebuilt. This runs inside the migration transaction with
-- foreign keys enforced: dropping the old table cascades to its request tables, so their rows are
-- held aside and restored once the new table carries the same identifiers.
CREATE TEMP TABLE held_alignment_requests AS SELECT * FROM spaces_media_alignment_requests;
CREATE TEMP TABLE held_diarization_requests AS SELECT * FROM spaces_media_diarization_requests;

CREATE TABLE spaces_enrichment_jobs_next (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  job_type TEXT NOT NULL CHECK (job_type IN ('analyze_visual', 'analyze_media', 'transcribe_media', 'align_media_transcript', 'diarize_media_transcript', 'parse_document', 'recognize_text')),
  status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'succeeded', 'failed', 'cancelled', 'awaiting_configuration')),
  source_revision TEXT NOT NULL,
  execution_target TEXT CHECK (execution_target IS NULL OR json_valid(execution_target)),
  attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
  error TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE
);

INSERT INTO spaces_enrichment_jobs_next
  (id, space_id, node_id, job_type, status, source_revision, execution_target, attempts, error, created_at, updated_at)
  SELECT id, space_id, node_id, job_type, status, source_revision, execution_target, attempts, error, created_at, updated_at
  FROM spaces_enrichment_jobs;

DROP TABLE spaces_enrichment_jobs;
ALTER TABLE spaces_enrichment_jobs_next RENAME TO spaces_enrichment_jobs;

CREATE UNIQUE INDEX spaces_enrichment_jobs_unique_pending
  ON spaces_enrichment_jobs(space_id, node_id, job_type, source_revision)
  WHERE status IN ('pending', 'running', 'awaiting_configuration');
CREATE INDEX spaces_enrichment_jobs_status ON spaces_enrichment_jobs(status, updated_at);
CREATE INDEX spaces_enrichment_jobs_node ON spaces_enrichment_jobs(node_id);
CREATE INDEX spaces_enrichment_jobs_space ON spaces_enrichment_jobs(space_id);

INSERT INTO spaces_media_alignment_requests SELECT * FROM held_alignment_requests;
INSERT INTO spaces_media_diarization_requests SELECT * FROM held_diarization_requests;
DROP TABLE held_alignment_requests;
DROP TABLE held_diarization_requests;

-- Up to 0.1.0, macOS rendered every PDF picture upside down and at a fraction of its size: Core
-- Graphics stores bitmap rows top-down and the renderer flipped them again, and its drawing transform never scales
-- up, so the page filled one corner of its raster. Every picture a PDF coverage record holds came
-- from that renderer, and nothing in the record says which platform made it, so each PDF is parsed
-- once more. Live documents are queued here; a deleted one is parsed when it is restored.
-- Withdrawing the record releases its renders to the startup orphan collector.
INSERT INTO spaces_enrichment_jobs
  (id, space_id, node_id, job_type, status, source_revision, execution_target, attempts, error, created_at, updated_at)
  SELECT lower(hex(randomblob(16))), n.space_id, n.id, 'parse_document', 'pending',
         json_extract(n.data, '$.sourceRevision'), NULL, 0, NULL,
         CAST(strftime('%s', 'now') AS INTEGER) * 1000, CAST(strftime('%s', 'now') AS INTEGER) * 1000
  FROM spaces_nodes n
  WHERE n.kind = 'file' AND n.deleted_at IS NULL
    AND json_extract(n.data, '$.documentCoverage.parser') = 'pdf-inspector'
    AND json_extract(n.data, '$.sourceRevision') IS NOT NULL
    AND NOT EXISTS (
      SELECT 1 FROM spaces_enrichment_jobs j
      WHERE j.node_id = n.id AND j.job_type = 'parse_document'
        AND j.source_revision = json_extract(n.data, '$.sourceRevision')
        AND j.status IN ('pending', 'running', 'awaiting_configuration')
    );

UPDATE spaces_nodes
  SET data = json_remove(data, '$.documentCoverage')
  WHERE kind = 'file' AND json_extract(data, '$.documentCoverage.parser') = 'pdf-inspector';
