-- One pre-release report contract. Obsolete derived reports are reset, never upgraded.
-- Startup completes canonical revision/chunk cleanup before exposing the database.
UPDATE spaces_enrichment_jobs SET status='cancelled',error=NULL
WHERE job_type IN ('parse_document','analyze_document')
  AND node_id IN (
    SELECT id FROM spaces_nodes
    WHERE json_type(data,'$.documentCoverage') IS NOT NULL
      AND json_extract(data,'$.documentCoverage.schemaVersion') IS NOT 1
  );

DELETE FROM spaces_document_visual_checkpoints
WHERE job_id IN (SELECT id FROM spaces_enrichment_jobs WHERE status='cancelled');

INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,error,created_at,updated_at)
SELECT lower(hex(randomblob(16))),space_id,id,'parse_document','pending',
       json_extract(data,'$.sourceRevision'),0,NULL,updated_at,updated_at
FROM spaces_nodes
WHERE deleted_at IS NULL AND kind='file'
  AND NULLIF(json_extract(data,'$.sourceRevision'),'') IS NOT NULL
  AND json_type(data,'$.documentCoverage') IS NOT NULL
  AND json_extract(data,'$.documentCoverage.schemaVersion') IS NOT 1;
