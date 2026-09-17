-- Supersede the development-only all-page visual plan. Original files and body text remain.
-- Manifest reset and canonical revision recomputation are completed in Rust before pool exposure.
UPDATE spaces_enrichment_jobs SET status='cancelled',error=NULL
WHERE job_type IN ('parse_document','analyze_document')
  AND node_id IN (SELECT id FROM spaces_nodes WHERE
    json_extract(data,'$.documentCoverage.parser') IN
      ('pdf-inspector:1.20.0:page-v1','office-oxide:0.1.8:resources-v1'));

INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,error,created_at,updated_at)
SELECT lower(hex(randomblob(16))),space_id,id,'parse_document','pending',json_extract(data,'$.sourceRevision'),0,NULL,updated_at,updated_at
FROM spaces_nodes WHERE deleted_at IS NULL AND kind='file'
  AND NULLIF(json_extract(data,'$.sourceRevision'),'') IS NOT NULL
  AND json_extract(data,'$.documentCoverage.parser') IN
    ('pdf-inspector:1.20.0:page-v1','office-oxide:0.1.8:resources-v1');
