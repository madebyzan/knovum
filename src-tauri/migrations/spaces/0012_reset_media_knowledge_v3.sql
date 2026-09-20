-- Upgrade mediaKnowledge from v3 to v4 in-place.
--
-- v3 stored the waveform array inside mediaKnowledge.transcript.
-- v4 splits it out into a dedicated mediaAnalysis field (schemaVersion 1),
-- and removes waveform from the transcript object.
--
-- Step 1: synthesise mediaAnalysis for nodes that have v3 mediaKnowledge
--         with a non-empty waveform but no current (same-revision) mediaAnalysis.
--         Nodes with an empty or absent waveform are skipped here; the intent
--         logic will re-queue analyze_media for them after the upgrade in step 2.
UPDATE spaces_nodes
SET data = json_set(
  data,
  '$.mediaAnalysis',
  json_object(
    'schemaVersion', 1,
    'sourceRevision', json_extract(data, '$.mediaKnowledge.sourceRevision'),
    'durationMs',     json_extract(data, '$.mediaKnowledge.transcript.durationMs'),
    'waveform',       json_extract(data, '$.mediaKnowledge.transcript.waveform')
  )
)
WHERE json_extract(data, '$.mediaKnowledge.schemaVersion') = 3
  AND json_type(data, '$.mediaKnowledge.transcript.waveform') = 'array'
  AND json_array_length(data, '$.mediaKnowledge.transcript.waveform') > 0
  AND (
    json_type(data, '$.mediaAnalysis') IS NULL
    OR json_extract(data, '$.mediaAnalysis.sourceRevision')
         IS NOT json_extract(data, '$.mediaKnowledge.sourceRevision')
  );

-- Step 2: remove waveform from transcript and bump schemaVersion to 4.
-- This applies to all remaining v3 nodes regardless of waveform presence.
-- Nodes without a valid mediaAnalysis will be re-queued by the intent logic.
UPDATE spaces_nodes
SET data = json_set(
  json_remove(data, '$.mediaKnowledge.transcript.waveform'),
  '$.mediaKnowledge.schemaVersion', 4
)
WHERE json_extract(data, '$.mediaKnowledge.schemaVersion') = 3;
