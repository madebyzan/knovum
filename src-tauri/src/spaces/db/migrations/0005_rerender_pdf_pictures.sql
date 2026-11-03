-- macOS rendered every PDF picture upside down and at a fraction of its size: Core Graphics stores
-- bitmap rows top-down and the renderer flipped them again, and its drawing transform never scales
-- up, so the page filled one corner of its raster. Every picture a PDF coverage record holds came
-- from that renderer, and nothing in the record says which platform made it, so each PDF is parsed
-- once more.
-- Live documents are queued here; a deleted one is parsed when it is restored. Withdrawing the record
-- releases its renders to the startup orphan collector.
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
