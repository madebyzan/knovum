-- Prompt-guided cloning keeps its style guidance in design_description on reference voice
-- candidates, which the 0001 CHECK reserved for designed candidates alone; the synthesis worker,
-- the candidate publication and the promotion into a voice revision all carry it there, so the
-- audition run of a prompt-guided clone failed the frozen CHECK at publication. SQLite cannot
-- alter a CHECK, so the table is rebuilt around the relaxed rule. Stored rows were written under
-- the stricter rule, so every one of them carries over unchanged.
CREATE TABLE spaces_voice_candidates_next (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL UNIQUE,
  source_kind TEXT NOT NULL CHECK (source_kind IN ('reference','designed')),
  language TEXT NOT NULL CHECK (length(trim(language)) > 0),
  sample_text TEXT NOT NULL CHECK (length(trim(sample_text)) BETWEEN 1 AND 500),
  design_description TEXT,
  reference_text TEXT,
  input_source_revision TEXT CHECK (input_source_revision IS NULL OR (length(input_source_revision)=71 AND substr(input_source_revision,1,7)='sha256:')),
  input_byte_length INTEGER CHECK (input_byte_length IS NULL OR input_byte_length BETWEEN 1 AND 134217728),
  reference_asset_path TEXT NOT NULL UNIQUE,
  reference_sha256 TEXT NOT NULL CHECK (length(reference_sha256)=64),
  reference_byte_length INTEGER NOT NULL CHECK (reference_byte_length > 44),
  preview_asset_path TEXT NOT NULL UNIQUE,
  preview_sha256 TEXT NOT NULL CHECK (length(preview_sha256)=64),
  preview_sample_rate INTEGER NOT NULL CHECK (preview_sample_rate > 0),
  preview_sample_count INTEGER NOT NULL CHECK (preview_sample_count > 0),
  model_spec_id TEXT NOT NULL CHECK (length(trim(model_spec_id)) > 0),
  seed INTEGER NOT NULL,
  synthesis_config TEXT NOT NULL CHECK (json_valid(synthesis_config) AND json_type(synthesis_config)='object'),
  authorization TEXT NOT NULL CHECK (json_valid(authorization) AND json_type(authorization)='object'),
  status TEXT NOT NULL CHECK (status IN ('ready','selected')),
  auditioned_at INTEGER,
  -- The selected profile and revision are the identities this candidate was promoted into,
  -- kept for audit after the voice itself is archived or replaced.
  selected_voice_profile_id TEXT,
  selected_voice_revision_id TEXT,
  selected_at INTEGER,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE,
  CHECK (id=run_id),
  CHECK (auditioned_at IS NULL OR auditioned_at>=created_at),
  CHECK (selected_at IS NULL OR selected_at>=created_at),
  CHECK ((source_kind='designed' AND design_description IS NOT NULL AND reference_text IS NOT NULL
          AND input_source_revision IS NULL AND input_byte_length IS NULL)
      OR (source_kind='reference' AND input_source_revision IS NOT NULL AND input_byte_length IS NOT NULL)),
  CHECK ((status='ready' AND selected_voice_profile_id IS NULL AND selected_voice_revision_id IS NULL AND selected_at IS NULL)
      OR (status='selected' AND auditioned_at IS NOT NULL AND selected_voice_profile_id IS NOT NULL
          AND selected_voice_revision_id IS NOT NULL AND selected_at IS NOT NULL)
)
);

INSERT INTO spaces_voice_candidates_next(
  id,run_id,source_kind,language,sample_text,design_description,reference_text,
  input_source_revision,input_byte_length,reference_asset_path,reference_sha256,reference_byte_length,
  preview_asset_path,preview_sha256,preview_sample_rate,preview_sample_count,model_spec_id,seed,
  synthesis_config,authorization,status,auditioned_at,selected_voice_profile_id,
  selected_voice_revision_id,selected_at,created_at)
SELECT id,run_id,source_kind,language,sample_text,design_description,reference_text,
  input_source_revision,input_byte_length,reference_asset_path,reference_sha256,reference_byte_length,
  preview_asset_path,preview_sha256,preview_sample_rate,preview_sample_count,model_spec_id,seed,
  synthesis_config,authorization,status,auditioned_at,selected_voice_profile_id,
  selected_voice_revision_id,selected_at,created_at
FROM spaces_voice_candidates;

DROP TABLE spaces_voice_candidates;
ALTER TABLE spaces_voice_candidates_next RENAME TO spaces_voice_candidates;

CREATE INDEX spaces_voice_candidates_library ON spaces_voice_candidates(status, created_at DESC);
