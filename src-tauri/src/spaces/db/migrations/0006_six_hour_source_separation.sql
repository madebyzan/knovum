-- Source separation accepted ten minutes of media because Spleeter decoded the whole source into
-- memory. HTDemucs streams the source in overlapping blocks, so the limit is now six hours: the most
-- a stereo 16-bit WAV stem holds at 44.1 kHz. SQLite cannot alter a CHECK, so the table is rebuilt
-- around the new bound; every stored separation was written under the stricter one and carries
-- over unchanged, including the model it was made with.
CREATE TABLE spaces_speech_source_separations_next (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  run_id TEXT NOT NULL UNIQUE,
  source_node_id TEXT NOT NULL,
  source_asset_path TEXT NOT NULL,
  source_revision TEXT NOT NULL CHECK (length(source_revision)=71 AND substr(source_revision,1,7)='sha256:'),
  source_duration_ms INTEGER NOT NULL CHECK (source_duration_ms > 0 AND source_duration_ms <= 21600000),
  model_spec_id TEXT NOT NULL CHECK (length(trim(model_spec_id)) > 0),
  background_asset_path TEXT NOT NULL UNIQUE,
  vocals_asset_path TEXT NOT NULL UNIQUE,
  vocals_sha256 TEXT NOT NULL CHECK (length(vocals_sha256)=64),
  background_sha256 TEXT NOT NULL CHECK (length(background_sha256)=64),
  background_sample_rate INTEGER NOT NULL CHECK (background_sample_rate > 0),
  background_sample_count INTEGER NOT NULL CHECK (background_sample_count > 0),
  background_duration_ms INTEGER NOT NULL CHECK (
    background_duration_ms=(background_sample_count * 1000 + background_sample_rate - 1) / background_sample_rate
  ),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE,
  FOREIGN KEY (source_node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE,
  CHECK (id=run_id)
);

INSERT INTO spaces_speech_source_separations_next(
  id,space_id,run_id,source_node_id,source_asset_path,source_revision,source_duration_ms,
  model_spec_id,background_asset_path,vocals_asset_path,vocals_sha256,background_sha256,
  background_sample_rate,background_sample_count,background_duration_ms,created_at)
SELECT id,space_id,run_id,source_node_id,source_asset_path,source_revision,source_duration_ms,
  model_spec_id,background_asset_path,vocals_asset_path,vocals_sha256,background_sha256,
  background_sample_rate,background_sample_count,background_duration_ms,created_at
FROM spaces_speech_source_separations;

DROP TABLE spaces_speech_source_separations;
ALTER TABLE spaces_speech_source_separations_next RENAME TO spaces_speech_source_separations;

CREATE INDEX spaces_speech_source_separations_space ON spaces_speech_source_separations(space_id, created_at DESC);
CREATE INDEX spaces_speech_source_separations_source ON spaces_speech_source_separations(source_node_id);
