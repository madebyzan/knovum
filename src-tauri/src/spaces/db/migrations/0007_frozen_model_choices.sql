-- Every user-triggered model choice is frozen when it is submitted and preserved on retry. The
-- transcription request joins the alignment and diarization requests in owning the model its job
-- runs, so a retry or a restart reads the frozen choice instead of today's Settings. Existing
-- transcription jobs were all started with the shipped default, whisperSmall, and carry it forward.
CREATE TABLE spaces_media_transcription_requests (
  job_id TEXT PRIMARY KEY NOT NULL,
  model_spec_id TEXT NOT NULL CHECK (length(model_spec_id) BETWEEN 1 AND 160),
  FOREIGN KEY (job_id) REFERENCES spaces_enrichment_jobs(id) ON DELETE CASCADE
);

INSERT INTO spaces_media_transcription_requests(job_id, model_spec_id)
SELECT j.id, '"whisperSmall"'
FROM spaces_enrichment_jobs j
WHERE j.job_type = 'transcribe_media'
  AND NOT EXISTS (
    SELECT 1 FROM spaces_media_transcription_requests r WHERE r.job_id = j.id
  );

-- Subtitle derivations now require the transcription model in their action parameters. Active
-- add-video-subtitles runs were accepted before the choice existed, so the shipped default is
-- frozen into both the run input and the step input a retry reads.
UPDATE spaces_runs
SET input_snapshot = json_insert(input_snapshot, '$.parameters.transcriptionModel', 'whisperSmall')
WHERE kind = 'action'
  AND json_extract(input_snapshot, '$.actionId') = 'add-video-subtitles'
  AND completed_at IS NULL
  AND json_extract(input_snapshot, '$.parameters.transcriptionModel') IS NULL;

UPDATE spaces_run_steps
SET resolved_input = json_insert(resolved_input, '$.parameters.transcriptionModel', 'whisperSmall')
WHERE node_kind = 'action'
  AND json_extract(resolved_input, '$.actionId') = 'add-video-subtitles'
  AND json_extract(resolved_input, '$.parameters.transcriptionModel') IS NULL
  AND run_id IN (SELECT id FROM spaces_runs WHERE completed_at IS NULL);

-- Reference voice candidates now carry the models that validate and isolate the reference. Active
-- candidate steps were frozen before those choices existed, so the shipped defaults are added to
-- the stored configuration a retry reads.
UPDATE spaces_run_steps
SET configuration = json_insert(configuration,
    '$.request.source.referenceValidationModel', 'whisperSmall',
    '$.request.source.isolationModel', 'htDemucs')
WHERE node_kind = 'speech_voice_candidate'
  AND json_extract(configuration, '$.request.source.kind') = 'reference'
  AND run_id IN (SELECT id FROM spaces_runs WHERE completed_at IS NULL);

-- Video dubbing now names the model its automatic background separation runs. Every stored dubbing
-- revision carries the shipped default forward so the required choice parses on the next assembly.
UPDATE spaces_speech_project_revisions
SET output_config = json_insert(output_config, '$.autoBackgroundSeparationModel', 'htDemucs')
WHERE project_id IN (SELECT id FROM spaces_speech_projects WHERE project_type = 'video_dubbing')
  AND json_extract(output_config, '$.autoBackgroundSeparationModel') IS NULL;
