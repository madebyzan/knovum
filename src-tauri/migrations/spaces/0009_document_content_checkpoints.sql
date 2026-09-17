-- Intermediate visuals belong to their existing job, not the canonical node knowledge revision.
CREATE TABLE spaces_document_visual_checkpoints (
  job_id TEXT NOT NULL REFERENCES spaces_enrichment_jobs(id) ON DELETE CASCADE,
  occurrence_id TEXT NOT NULL,
  occurrence TEXT NOT NULL CHECK(json_valid(occurrence)),
  resource TEXT CHECK(resource IS NULL OR json_valid(resource)),
  PRIMARY KEY(job_id,occurrence_id)
);

-- Explicitly discard obsolete development-only derived reports and reparse originals.
-- Rust completes the report reset and canonical revision recomputation before exposing the pool.
UPDATE spaces_enrichment_jobs SET status='cancelled',error=NULL
WHERE job_type IN ('parse_document','analyze_document')
  AND node_id IN (SELECT id FROM spaces_nodes WHERE
    json_extract(data,'$.documentCoverage.parser') IN
      ('pdf-inspector:1.20.0:regions-v1','office-oxide:0.1.8:resources-v2'));

INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,error,created_at,updated_at)
SELECT lower(hex(randomblob(16))),space_id,id,'parse_document','pending',json_extract(data,'$.sourceRevision'),0,NULL,updated_at,updated_at
FROM spaces_nodes WHERE deleted_at IS NULL AND kind='file'
  AND NULLIF(json_extract(data,'$.sourceRevision'),'') IS NOT NULL
  AND json_extract(data,'$.documentCoverage.parser') IN
    ('pdf-inspector:1.20.0:regions-v1','office-oxide:0.1.8:resources-v2');
