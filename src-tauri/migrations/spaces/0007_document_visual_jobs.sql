-- Replace the obsolete development-only PDF report, keeping originals and text.
UPDATE spaces_nodes SET data=json_remove(data,'$.documentCoverage')
WHERE json_extract(data,'$.documentCoverage.parser')='pdf-inspector:1.20.0';

CREATE TABLE spaces_enrichment_jobs_next (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  job_type TEXT NOT NULL CHECK (job_type IN ('embed_text','analyze_image','transcribe_media','analyze_video_first_frame','embed_media_transcript','parse_document','analyze_document')),
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

-- Reparse existing documents through the same durable jobs used for imports.
INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,error,created_at,updated_at)
SELECT lower(hex(randomblob(16))),space_id,id,'parse_document','pending',json_extract(data,'$.sourceRevision'),0,NULL,updated_at,updated_at
FROM spaces_nodes
WHERE deleted_at IS NULL AND kind='file'
  AND NULLIF(json_extract(data,'$.sourceRevision'),'') IS NOT NULL
  AND (lower(json_extract(data,'$.fileName')) GLOB '*.pdf'
    OR lower(json_extract(data,'$.fileName')) GLOB '*.doc'
    OR lower(json_extract(data,'$.fileName')) GLOB '*.docx'
    OR lower(json_extract(data,'$.fileName')) GLOB '*.xls'
    OR lower(json_extract(data,'$.fileName')) GLOB '*.xlsx'
    OR lower(json_extract(data,'$.fileName')) GLOB '*.ppt'
    OR lower(json_extract(data,'$.fileName')) GLOB '*.pptx');
