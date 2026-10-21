-- Current pre-release Spaces v1 baseline. Obsolete databases are rejected, never reinterpreted.

CREATE TABLE spaces (
  id TEXT PRIMARY KEY NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('inbox', 'space')),
  title TEXT NOT NULL,
  archived_at INTEGER,
  favorite INTEGER NOT NULL DEFAULT 0 CHECK (favorite IN (0, 1)),
  thumbnail_path TEXT,
  last_opened_at INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE spaces_object_types (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT,
  name TEXT NOT NULL,
  icon TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  fields TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(fields)),
  system INTEGER NOT NULL CHECK (system IN (0, 1)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  CHECK ((system = 1 AND space_id IS NULL AND id LIKE 'system:%') OR (system = 0 AND space_id IS NOT NULL AND id NOT LIKE 'system:%'))
);

CREATE TABLE spaces_nodes (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (
    kind IN (
      'note', 'text', 'website', 'markdown', 'image', 'video', 'audio', 'file', 'code',
      'folder', 'archive', 'credential', 'object', 'portal'
    )
  ),
  object_type_id TEXT,
  initial_x REAL NOT NULL,
  initial_y REAL NOT NULL,
  data TEXT NOT NULL CHECK (json_valid(data)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (object_type_id) REFERENCES spaces_object_types(id) ON DELETE RESTRICT,
  CHECK (
    (kind = 'object' AND object_type_id IS NOT NULL) OR
    (kind <> 'object' AND object_type_id IS NULL)
  ),
  CHECK (json_extract(data, '$.kind') = kind),
  CHECK (
    kind <> 'object' OR json_extract(data, '$.objectTypeId') = object_type_id
  )
);

CREATE TABLE spaces_structure_nodes (
  id TEXT PRIMARY KEY NOT NULL CHECK (length(id) > 0),
  space_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('content', 'group', 'section')),
  node_id TEXT,
  parent_id TEXT,
  order_index INTEGER NOT NULL CHECK (order_index >= 0),
  title TEXT,
  locked INTEGER NOT NULL DEFAULT 0 CHECK (locked IN (0, 1)),
  deleted_at INTEGER,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (space_id, node_id) REFERENCES spaces_nodes(space_id, id)
    ON DELETE NO ACTION DEFERRABLE INITIALLY DEFERRED,
  FOREIGN KEY (space_id, parent_id) REFERENCES spaces_structure_nodes(space_id, id)
    ON DELETE NO ACTION DEFERRABLE INITIALLY DEFERRED,
  CHECK (parent_id IS NULL OR parent_id <> id),
  CHECK (
    (kind = 'content' AND node_id IS NOT NULL AND title IS NULL) OR
    (kind IN ('group', 'section') AND node_id IS NULL AND title IS NOT NULL)
  )
);

CREATE VIRTUAL TABLE spaces_structure_title_fts USING fts5(
  structure_node_id UNINDEXED, space_id UNINDEXED, kind UNINDEXED, title, tokenize='unicode61'
);

CREATE TABLE spaces_structure_layouts (
  space_id TEXT NOT NULL,
  structure_node_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('content', 'group', 'section')),
  data TEXT NOT NULL CHECK (
    json_valid(data) AND json_type(data) = 'object' AND
    COALESCE(json_extract(data, '$.kind'), '') = kind AND
    COALESCE(json_extract(data, '$.structureNodeId'), '') = structure_node_id AND
    (kind <> 'content' OR COALESCE(json_extract(data, '$.sizingMode'), '') IN ('content', 'frame')) AND
    (kind <> 'section' OR (
      json_type(data, '$.size.width') IN ('integer', 'real') AND
      json_type(data, '$.size.height') IN ('integer', 'real') AND
      json_extract(data, '$.size.width') >= 240 AND
      json_extract(data, '$.size.height') >= 160
    ))
  ),
  PRIMARY KEY (space_id, structure_node_id),
  FOREIGN KEY (space_id, structure_node_id, kind)
    REFERENCES spaces_structure_nodes(space_id, id, kind) ON DELETE CASCADE
);

CREATE TABLE spaces_edit_state (
  space_id TEXT PRIMARY KEY NOT NULL,
  revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0),
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_canvas_view_state (
  space_id TEXT PRIMARY KEY NOT NULL,
  x REAL NOT NULL,
  y REAL NOT NULL,
  zoom REAL NOT NULL CHECK (zoom >= 0.05 AND zoom <= 16),
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_edit_receipts (
  mutation_id TEXT PRIMARY KEY NOT NULL CHECK (length(mutation_id) > 0),
  space_id TEXT NOT NULL,
  request_digest TEXT NOT NULL CHECK (length(request_digest) = 64),
  applied_revision INTEGER NOT NULL CHECK (applied_revision > 0),
  selection_suggestion TEXT NOT NULL CHECK (
    json_valid(selection_suggestion) AND json_type(selection_suggestion) = 'array'
  ),
  committed_projection TEXT NOT NULL CHECK (json_valid(committed_projection)),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

-- One edit remembers the rows it touched, not the state it produced: `patch` holds the before and
-- after value of every changed structure row, layout, relation style, relation lifecycle and node
-- presence, and nothing else. An edit of one card therefore costs the same in a Space of ten
-- thousand cards as in a Space of ten.
CREATE TABLE spaces_edit_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  space_id TEXT NOT NULL,
  patch TEXT NOT NULL CHECK (json_valid(patch) AND json_type(patch) = 'object'),
  history_group_id TEXT,
  undone INTEGER NOT NULL DEFAULT 0 CHECK (undone IN (0,1)),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_scenes (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  title TEXT NOT NULL,
  viewport_x REAL NOT NULL,
  viewport_y REAL NOT NULL,
  viewport_zoom REAL NOT NULL,
  thumbnail_path TEXT,
  order_index INTEGER NOT NULL DEFAULT 0,
  focus_structure_node_ids TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(focus_structure_node_ids)),
  relation_kinds TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(relation_kinds)),
  relation_statuses TEXT NOT NULL DEFAULT '["active","contested"]' CHECK (json_valid(relation_statuses)),
  query_filter TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(query_filter)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_relations (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  source_id TEXT NOT NULL,
  target_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (
    kind IN ('reference', 'related', 'supports', 'contradicts', 'depends-on',
             'supersedes', 'answers', 'uses', 'derives-from', 'caused-by')
  ),
  rationale TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'contested', 'invalidated')),
  status_before_invalidation TEXT CHECK (status_before_invalidation IN ('active', 'contested')),
  confidence REAL CHECK (confidence IS NULL OR (confidence >= 0 AND confidence <= 1)),
  actor_kind TEXT NOT NULL DEFAULT 'human' CHECK (actor_kind IN ('human', 'builtin_agent', 'external_agent', 'system')),
  actor_id TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (source_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (target_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE,
  CHECK (source_id <> target_id),
  CHECK ((status = 'invalidated' AND status_before_invalidation IS NOT NULL) OR
         (status <> 'invalidated' AND status_before_invalidation IS NULL))
);

CREATE TABLE spaces_relation_styles (
  space_id TEXT NOT NULL,
  relation_id TEXT NOT NULL,
  color TEXT NOT NULL CHECK (color IN ('blue', 'green', 'amber', 'red', 'purple')),
  PRIMARY KEY (space_id, relation_id),
  FOREIGN KEY (space_id, relation_id) REFERENCES spaces_relations(space_id, id) ON DELETE CASCADE
);

CREATE TABLE spaces_knowledge_derivations (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  output_node_id TEXT NOT NULL,
  action_kind TEXT NOT NULL CHECK (length(trim(action_kind)) > 0),
  action_version INTEGER NOT NULL CHECK (action_version > 0),
  engine TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed')),
  error TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  completed_at INTEGER,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (output_node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE,
  CHECK ((status = 'completed' AND completed_at IS NOT NULL AND error = '') OR
         (status = 'failed' AND completed_at IS NOT NULL AND length(error) > 0) OR
         (status = 'running' AND completed_at IS NULL AND error = ''))
);

CREATE TABLE spaces_knowledge_derivation_inputs (
  derivation_id TEXT NOT NULL,
  position INTEGER NOT NULL CHECK (position >= 0),
  node_id TEXT,
  source_identity TEXT NOT NULL CHECK (length(source_identity) > 0),
  source_snapshot TEXT NOT NULL CHECK (
    json_valid(source_snapshot) AND json_type(source_snapshot) = 'object'
    AND json_type(source_snapshot, '$.title') IS 'text'
    AND json_type(source_snapshot, '$.kind') IS 'text'
    AND json_type(source_snapshot, '$.originalUrl') IN ('text', 'null')
    AND json_type(source_snapshot, '$.platform') IN ('text', 'null')
  ),
  role TEXT NOT NULL CHECK (role IN ('primary', 'source', 'context', 'style-reference')),
  source_revision TEXT NOT NULL,
  revision_basis TEXT NOT NULL CHECK (revision_basis IN ('asset', 'rendered', 'knowledge')),
  input_revision TEXT NOT NULL CHECK (length(input_revision) > 0),
  input_snapshot TEXT NOT NULL CHECK (json_valid(input_snapshot)),
  PRIMARY KEY (derivation_id, position),
  UNIQUE (derivation_id, source_identity, role),
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES spaces_nodes(id) ON DELETE SET NULL
);

CREATE TABLE spaces_agent_artifact_sessions (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL UNIQUE,
  space_id TEXT NOT NULL,
  runtime_kind TEXT NOT NULL,
  state TEXT NOT NULL CHECK (state IN ('running','completed','failed','cancelled')),
  error TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  completed_at INTEGER,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE
);

CREATE TABLE spaces_reserved_outputs (
  node_id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  state_field TEXT NOT NULL CHECK (state_field IN ('processingState', 'derivationState')),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE
);

CREATE TABLE spaces_grounded_documents (
  derivation_id TEXT PRIMARY KEY NOT NULL,
  citations TEXT NOT NULL CHECK (json_valid(citations) AND json_type(citations) = 'array'),
  execution_source TEXT NOT NULL CHECK (json_valid(execution_source)),
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE
);

CREATE TABLE spaces_generated_images (
  derivation_id TEXT PRIMARY KEY NOT NULL,
  intent TEXT NOT NULL,
  aspect_ratio TEXT NOT NULL CHECK (length(trim(aspect_ratio)) > 0),
  execution_source TEXT NOT NULL CHECK (json_valid(execution_source)),
  revised_prompt TEXT,
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE
);

CREATE TABLE spaces_generated_videos (
  derivation_id TEXT PRIMARY KEY NOT NULL,
  prompt TEXT NOT NULL,
  aspect_ratio TEXT NOT NULL CHECK (length(trim(aspect_ratio)) > 0),
  duration_seconds INTEGER NOT NULL CHECK (duration_seconds > 0),
  execution_source TEXT NOT NULL CHECK (json_valid(execution_source)),
  operation_id TEXT,
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE
);

CREATE TABLE spaces_provenance (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  target_kind TEXT NOT NULL CHECK (target_kind IN ('node', 'relation')),
  target_id TEXT NOT NULL,
  source_node_id TEXT NOT NULL,
  locator TEXT NOT NULL CHECK (json_valid(locator)),
  source_revision TEXT NOT NULL,
  quote TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'superseded', 'invalidated')),
  actor_kind TEXT NOT NULL CHECK (actor_kind IN ('human', 'builtin_agent', 'external_agent', 'system')),
  actor_id TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (source_node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE
);

CREATE TABLE spaces_knowledge_events (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  entity_kind TEXT NOT NULL CHECK (entity_kind IN ('node', 'relation', 'provenance', 'patch', 'type')),
  entity_id TEXT NOT NULL,
  event_kind TEXT NOT NULL CHECK (
    event_kind IN ('created', 'updated', 'drafted', 'accepted', 'contested', 'superseded', 'invalidated', 'opened', 'resolved', 'restored', 'applied', 'rejected', 'moved')
  ),
  actor_kind TEXT NOT NULL CHECK (actor_kind IN ('human', 'builtin_agent', 'external_agent', 'system')),
  actor_id TEXT,
  payload TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(payload)),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_knowledge_chunks (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  origin TEXT NOT NULL CHECK (origin IN ('node', 'source', 'media', 'derived')),
  kind TEXT NOT NULL CHECK (
    kind IN ('node', 'text', 'heading', 'page', 'slide', 'sheet', 'table', 'transcript', 'chapter', 'code', 'image_description', 'video_cover')
  ),
  source_revision TEXT NOT NULL,
  text TEXT NOT NULL,
  locator TEXT NOT NULL CHECK (json_valid(locator)),
  metadata TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(metadata)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE spaces_knowledge_fts USING fts5(
  chunk_id UNINDEXED, space_id UNINDEXED, node_id UNINDEXED, text, tokenize = 'trigram'
);

CREATE TABLE spaces_retrieval_generations (
  id TEXT PRIMARY KEY NOT NULL CHECK (length(id) > 0),
  space_id TEXT NOT NULL,
  embedding_spec_id TEXT NOT NULL CHECK (length(embedding_spec_id) > 0),
  status TEXT NOT NULL CHECK (status IN ('building', 'active', 'retired')),
  created_at INTEGER NOT NULL,
  activated_at INTEGER,
  cleanup_started_at INTEGER,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  CHECK ((status = 'active' AND activated_at IS NOT NULL) OR
         (status <> 'active' AND activated_at IS NULL)),
  CHECK (cleanup_started_at IS NULL OR (status='retired' AND cleanup_started_at>=created_at))
);

CREATE TABLE spaces_chunk_embeddings (
  generation_id TEXT NOT NULL,
  chunk_id TEXT NOT NULL,
  source_revision TEXT NOT NULL,
  embedding_spec_id TEXT NOT NULL CHECK (length(embedding_spec_id) > 0),
  dimensions INTEGER NOT NULL CHECK (dimensions > 0),
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (generation_id, chunk_id),
  FOREIGN KEY (generation_id) REFERENCES spaces_retrieval_generations(id) ON DELETE CASCADE,
  FOREIGN KEY (chunk_id) REFERENCES spaces_knowledge_chunks(id) ON DELETE CASCADE
);

-- A tombstone queue, deliberately without foreign keys. Each row asks the retrieval writer to
-- delete the vectors of a Space or a node from LanceDB, so it must outlive the very rows it names:
-- a cascade would erase the instruction together with its subject and leave the vectors in the
-- index forever. Rows are removed on settlement, not by referential integrity.
CREATE TABLE spaces_retrieval_invalidations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  space_id TEXT NOT NULL CHECK (length(space_id) > 0),
  node_id TEXT CHECK (node_id IS NULL OR length(node_id) > 0),
  attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
  last_error TEXT,
  created_at INTEGER NOT NULL
);

CREATE TABLE spaces_enrichment_jobs (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  job_type TEXT NOT NULL CHECK (job_type IN ('analyze_visual', 'analyze_media', 'transcribe_media', 'align_media_transcript', 'diarize_media_transcript', 'parse_document')),
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

CREATE TABLE spaces_media_alignment_requests (
  job_id TEXT PRIMARY KEY NOT NULL,
  model_spec_id TEXT NOT NULL CHECK (length(model_spec_id) BETWEEN 1 AND 160),
  expected_transcript_revision INTEGER NOT NULL CHECK (expected_transcript_revision > 0),
  segment_ids TEXT NOT NULL CHECK (json_valid(segment_ids) AND json_type(segment_ids) = 'array'),
  FOREIGN KEY (job_id) REFERENCES spaces_enrichment_jobs(id) ON DELETE CASCADE
);

CREATE TABLE spaces_media_diarization_requests (
  job_id TEXT PRIMARY KEY NOT NULL,
  model_spec_id TEXT NOT NULL CHECK (length(model_spec_id) BETWEEN 1 AND 160),
  expected_transcript_revision INTEGER NOT NULL CHECK (expected_transcript_revision > 0),
  minimum_speakers INTEGER CHECK (minimum_speakers BETWEEN 1 AND 32),
  maximum_speakers INTEGER CHECK (maximum_speakers BETWEEN 1 AND 32),
  CHECK (minimum_speakers IS NULL OR maximum_speakers IS NULL OR minimum_speakers <= maximum_speakers),
  FOREIGN KEY (job_id) REFERENCES spaces_enrichment_jobs(id) ON DELETE CASCADE
);

CREATE TABLE spaces_authorization_grants (
  id TEXT PRIMARY KEY NOT NULL,
  principal_kind TEXT NOT NULL CHECK (principal_kind IN ('user','builtin_agent','external_agent','automation')),
  principal_id TEXT NOT NULL,
  scope_kind TEXT NOT NULL CHECK (scope_kind IN ('space','application')),
  home_space_id TEXT,
  read_space_ids TEXT NOT NULL CHECK (json_valid(read_space_ids) AND json_type(read_space_ids) = 'array'),
  write_space_ids TEXT NOT NULL CHECK (json_valid(write_space_ids) AND json_type(write_space_ids) = 'array'),
  action_ids TEXT NOT NULL CHECK (json_valid(action_ids) AND json_type(action_ids) = 'array'),
  valid_until INTEGER NOT NULL,
  revoked_at INTEGER,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (home_space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  CHECK ((scope_kind='space' AND home_space_id IS NOT NULL AND length(trim(home_space_id))>0)
    OR (scope_kind='application' AND home_space_id IS NULL AND principal_kind='user'
      AND json_array_length(read_space_ids)=0 AND json_array_length(write_space_ids)=0
      AND json_array_length(action_ids)=0))
);

CREATE TABLE spaces_action_invocations (
  id TEXT PRIMARY KEY NOT NULL,
  principal_kind TEXT NOT NULL,
  principal_id TEXT NOT NULL,
  space_id TEXT NOT NULL,
  action_id TEXT NOT NULL,
  action_version INTEGER NOT NULL CHECK (action_version > 0),
  idempotency_key TEXT NOT NULL,
  request_digest TEXT NOT NULL CHECK (length(request_digest) = 64),
  outcome_kind TEXT NOT NULL CHECK (outcome_kind IN ('projection','committed_receipt','accepted_run')),
  outcome TEXT NOT NULL CHECK (json_valid(outcome)),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  UNIQUE (principal_kind, principal_id, space_id, action_id, action_version, idempotency_key)
);

CREATE TABLE spaces_runs (
  id TEXT PRIMARY KEY NOT NULL,
  scope_kind TEXT NOT NULL CHECK (scope_kind IN ('space','application')),
  home_space_id TEXT,
  parent_run_id TEXT,
  retry_of_run_id TEXT,
  fork_of_run_id TEXT,
  principal_kind TEXT NOT NULL,
  principal_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('action','workflow','builtin_agent','external_agent','automation')),
  state TEXT NOT NULL CHECK (state IN ('queued','running','waiting','paused','succeeded','completed_with_errors','failed','cancelled')),
  state_version INTEGER NOT NULL DEFAULT 0 CHECK (state_version >= 0),
  execution_epoch INTEGER NOT NULL DEFAULT 0 CHECK (execution_epoch >= 0),
  wait_reason TEXT,
  policy_snapshot TEXT NOT NULL CHECK (json_valid(policy_snapshot)),
  input_snapshot TEXT NOT NULL CHECK (json_valid(input_snapshot)),
  budget TEXT NOT NULL CHECK (json_valid(budget)),
  input_tokens INTEGER NOT NULL DEFAULT 0 CHECK (input_tokens >= 0),
  output_tokens INTEGER NOT NULL DEFAULT 0 CHECK (output_tokens >= 0),
  total_tokens INTEGER NOT NULL DEFAULT 0 CHECK (total_tokens >= 0),
  step_attempts INTEGER NOT NULL DEFAULT 0 CHECK (step_attempts >= 0),
  expanded_steps INTEGER NOT NULL DEFAULT 0 CHECK (expanded_steps >= 0),
  output_bytes INTEGER NOT NULL DEFAULT 0 CHECK (output_bytes >= 0),
  cancel_requested_at INTEGER,
  pause_requested_at INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  completed_at INTEGER,
  FOREIGN KEY (home_space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (parent_run_id) REFERENCES spaces_runs(id) ON DELETE SET NULL,
  FOREIGN KEY (retry_of_run_id) REFERENCES spaces_runs(id) ON DELETE SET NULL,
  FOREIGN KEY (fork_of_run_id) REFERENCES spaces_runs(id) ON DELETE SET NULL,
  CHECK ((scope_kind='space' AND home_space_id IS NOT NULL AND length(trim(home_space_id))>0)
    OR (scope_kind='application' AND home_space_id IS NULL AND principal_kind='user'
      AND kind='workflow' AND parent_run_id IS NULL
      AND json_extract(input_snapshot,'$.workflow') IS 'speech_voice_candidate_v1'))
);

CREATE TABLE spaces_run_steps (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  definition_node_id TEXT NOT NULL,
  instance_path TEXT NOT NULL,
  node_kind TEXT NOT NULL,
  depends_on TEXT NOT NULL DEFAULT '[]'
    CHECK (json_valid(depends_on) AND json_type(depends_on) = 'array'),
  configuration TEXT NOT NULL DEFAULT '{}'
    CHECK (json_valid(configuration) AND json_type(configuration) = 'object'),
  state TEXT NOT NULL CHECK (state IN ('pending','ready','running','waiting','paused','succeeded','failed','skipped','blocked','cancelled')),
  state_version INTEGER NOT NULL DEFAULT 0 CHECK (state_version >= 0),
  resolved_input TEXT NOT NULL CHECK (json_valid(resolved_input)),
  output TEXT CHECK (output IS NULL OR json_valid(output)),
  wait_reason TEXT,
  available_at INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE,
  UNIQUE (run_id, instance_path)
);

CREATE TABLE spaces_run_attempts (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  step_instance_id TEXT NOT NULL,
  attempt_number INTEGER NOT NULL CHECK (attempt_number > 0),
  state TEXT NOT NULL CHECK (state IN ('claimed','executing','waiting_external','succeeded','failed','cancelled','outcome_unknown')),
  request_digest TEXT NOT NULL CHECK (length(request_digest) = 64),
  executor_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  external_task_id TEXT,
  execution_epoch INTEGER NOT NULL CHECK (execution_epoch >= 0),
  lease_until INTEGER,
  checkpoint TEXT CHECK (checkpoint IS NULL OR json_valid(checkpoint)),
  output TEXT CHECK (output IS NULL OR json_valid(output)),
  error_code TEXT,
  started_at INTEGER NOT NULL,
  finished_at INTEGER,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE,
  FOREIGN KEY (step_instance_id) REFERENCES spaces_run_steps(id) ON DELETE CASCADE,
  UNIQUE (step_instance_id, attempt_number),
  UNIQUE (executor_id, idempotency_key)
);

CREATE TABLE spaces_run_events (
  sequence INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id TEXT NOT NULL,
  step_instance_id TEXT,
  attempt_id TEXT,
  kind TEXT NOT NULL,
  payload TEXT NOT NULL CHECK (json_valid(payload)),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE
);

CREATE TABLE spaces_external_agent_conversations (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL UNIQUE,
  space_id TEXT NOT NULL,
  runtime_kind TEXT NOT NULL,
  runtime_session_id TEXT,
  session_capabilities TEXT CHECK (session_capabilities IS NULL OR json_valid(session_capabilities)),
  title TEXT NOT NULL,
  state TEXT NOT NULL CHECK (state IN ('connecting','connected','disconnected','closed')),
  output_publication_state TEXT NOT NULL DEFAULT 'pending' CHECK (output_publication_state IN ('pending','publishing','committed','failed')),
  output_publication_error TEXT,
  error TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  conversation_id TEXT REFERENCES spaces_conversations(id) ON DELETE CASCADE,
  branch_id TEXT,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_external_agent_turns (
  id TEXT PRIMARY KEY NOT NULL,
  conversation_id TEXT NOT NULL,
  state TEXT NOT NULL CHECK (state IN ('running','completed','failed','cancelled')),
  error TEXT,
  created_at INTEGER NOT NULL,
  completed_at INTEGER,
  message_id TEXT REFERENCES spaces_conversation_messages(id) ON DELETE SET NULL,
  FOREIGN KEY (conversation_id) REFERENCES spaces_external_agent_conversations(id) ON DELETE CASCADE
);

CREATE TABLE spaces_external_output_staging (
  run_id TEXT NOT NULL,
  path TEXT NOT NULL CHECK (length(path) > 0),
  role TEXT NOT NULL CHECK (role IN ('asset','preview','embedded')),
  created_at INTEGER NOT NULL,
  PRIMARY KEY (run_id, path),
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE
);

CREATE TABLE spaces_conversations (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  title TEXT NOT NULL,
  archived_at INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL, active_branch_id TEXT,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_conversation_drafts (
  space_id TEXT NOT NULL,
  conversation_id TEXT NOT NULL CHECK (length(conversation_id) <= 64),
  prompt TEXT NOT NULL CHECK (length(prompt) <= 1000000),
  attachments TEXT NOT NULL CHECK (json_valid(attachments) AND json_type(attachments) = 'array'),
  extensions TEXT NOT NULL CHECK (json_valid(extensions) AND json_type(extensions) = 'object'),
  reacquire_file_names TEXT NOT NULL
    CHECK (json_valid(reacquire_file_names) AND json_type(reacquire_file_names) = 'array'),
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (space_id, conversation_id),
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_conversation_messages (
  id TEXT PRIMARY KEY NOT NULL,
  conversation_id TEXT NOT NULL,
  branch_id TEXT NOT NULL,
  parent_message_id TEXT,
  role TEXT NOT NULL CHECK (role IN ('user','assistant','tool')),
  content TEXT NOT NULL CHECK (json_valid(content)),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (conversation_id) REFERENCES spaces_conversations(id) ON DELETE CASCADE,
  FOREIGN KEY (parent_message_id) REFERENCES spaces_conversation_messages(id) ON DELETE SET NULL
);

CREATE TABLE spaces_builtin_agent_execution_segments (
  id TEXT PRIMARY KEY NOT NULL,
  conversation_id TEXT NOT NULL,
  branch_id TEXT NOT NULL,
  execution_revision TEXT NOT NULL,
  graph_version TEXT NOT NULL,
  frozen_target TEXT NOT NULL CHECK (json_valid(frozen_target)),
  model_profile TEXT NOT NULL CHECK (json_valid(model_profile)),
  target_selection TEXT NOT NULL CHECK (json_valid(target_selection)),
  catalog_digest TEXT NOT NULL CHECK (length(catalog_digest) = 64),
  projection_checkpoint_id TEXT,
  checkpoint_deleted_at INTEGER,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (conversation_id) REFERENCES spaces_conversations(id) ON DELETE CASCADE
);

CREATE TABLE spaces_builtin_agent_tasks (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL UNIQUE,
  conversation_id TEXT NOT NULL,
  branch_id TEXT NOT NULL,
  grant_id TEXT NOT NULL,
  model_binding TEXT NOT NULL CHECK (json_valid(model_binding)),
  catalog_digest TEXT NOT NULL CHECK (length(catalog_digest) = 64),
  segment_id TEXT NOT NULL,
  input_id TEXT NOT NULL UNIQUE,
  input_accepted INTEGER NOT NULL DEFAULT 0 CHECK (input_accepted IN (0,1)),
  control_epoch INTEGER NOT NULL DEFAULT 0 CHECK (control_epoch >= 0),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL, error TEXT,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE,
  FOREIGN KEY (conversation_id) REFERENCES spaces_conversations(id) ON DELETE CASCADE,
  FOREIGN KEY (grant_id) REFERENCES spaces_authorization_grants(id) ON DELETE RESTRICT,
  FOREIGN KEY (segment_id) REFERENCES spaces_builtin_agent_execution_segments(id) ON DELETE CASCADE
);

CREATE TABLE spaces_builtin_agent_model_calls (
  id TEXT PRIMARY KEY NOT NULL,
  task_id TEXT NOT NULL,
  purpose TEXT NOT NULL CHECK (purpose IN ('main','research','context_summary')),
  graph_namespace TEXT NOT NULL,
  checkpoint_id TEXT NOT NULL,
  message_id TEXT,
  retry_authorized_at INTEGER,
  input_tokens INTEGER CHECK (input_tokens >= 0),
  output_tokens INTEGER CHECK (output_tokens >= 0),
  state TEXT NOT NULL CHECK (state IN ('requesting','received','committed','outcome_unknown','usage_unknown')),
  created_at INTEGER NOT NULL,
  completed_at INTEGER,
  FOREIGN KEY (task_id) REFERENCES spaces_builtin_agent_tasks(id) ON DELETE CASCADE
);

CREATE TABLE spaces_conversation_tool_operations (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  task_id TEXT,
  graph_namespace TEXT,
  checkpoint_id TEXT,
  message_id TEXT,
  provider_call_id TEXT NOT NULL,
  wire_name TEXT NOT NULL,
  source TEXT NOT NULL CHECK(json_valid(source)),
  invocation_id TEXT NOT NULL,
  attempt INTEGER NOT NULL DEFAULT 1,
  context TEXT NOT NULL CHECK(json_valid(context)),
  context_digest TEXT NOT NULL,
  read_only INTEGER NOT NULL CHECK(read_only IN (0,1)),
  approval TEXT CHECK(approval IS NULL OR approval IN ('allow','deny')),
  state TEXT NOT NULL DEFAULT 'prepared' CHECK(state IN ('prepared','dispatching','completed','failed','outcome_unknown')),
  resolution TEXT CHECK(resolution IS NULL OR resolution IN ('new_attempt','stop')),
  target_selection TEXT CHECK(target_selection IS NULL OR json_valid(target_selection)),
  output TEXT CHECK(output IS NULL OR json_valid(output)),
  created_at INTEGER NOT NULL,
  completed_at INTEGER,
  UNIQUE(invocation_id, attempt),
  FOREIGN KEY(run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE,
  FOREIGN KEY(task_id) REFERENCES spaces_builtin_agent_tasks(id) ON DELETE CASCADE
);
CREATE INDEX spaces_conversation_tool_operation_run ON spaces_conversation_tool_operations(run_id,state);

CREATE TABLE spaces_conversation_mcp_interactions (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  request TEXT NOT NULL CHECK(json_valid(request)),
  response TEXT CHECK(response IS NULL OR json_valid(response)),
  created_at INTEGER NOT NULL,
  FOREIGN KEY(run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE
);

CREATE TABLE spaces_builtin_agent_tool_admissions (
  id TEXT PRIMARY KEY NOT NULL,
  task_id TEXT NOT NULL,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (task_id) REFERENCES spaces_builtin_agent_tasks(id) ON DELETE CASCADE
);

CREATE TABLE spaces_conversation_message_projections (
  segment_id TEXT NOT NULL,
  provider_message_id TEXT NOT NULL,
  message_id TEXT NOT NULL UNIQUE,
  PRIMARY KEY (segment_id, provider_message_id),
  FOREIGN KEY (segment_id) REFERENCES spaces_builtin_agent_execution_segments(id) ON DELETE CASCADE,
  FOREIGN KEY (message_id) REFERENCES spaces_conversation_messages(id) ON DELETE CASCADE
);

-- The indexed passages a Run read, numbered once per Run (`C1`, `C2`, …) so an
-- answer's markers resolve to the exact chunk. Chunk and node ids are stored
-- identities, not references: the passage is kept as it was read.
CREATE TABLE spaces_run_citations (
  run_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL CHECK (ordinal > 0),
  chunk_id TEXT NOT NULL,
  source_revision TEXT NOT NULL,
  node_id TEXT NOT NULL,
  node_title TEXT NOT NULL CHECK (length(node_title) BETWEEN 1 AND 512),
  locator TEXT NOT NULL CHECK (json_valid(locator)),
  quote TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (run_id, ordinal),
  UNIQUE (run_id, chunk_id, source_revision),
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE
);

INSERT INTO "spaces_object_types" ("id","space_id","name","icon","description","fields","system","created_at","updated_at") VALUES ('system:claim',NULL,'Claim','lightbulb','An atomic assertion that can be supported, challenged, or superseded.','[{"key":"statement","label":"Statement","kind":"text","required":true}]',1,0,0);

INSERT INTO "spaces_object_types" ("id","space_id","name","icon","description","fields","system","created_at","updated_at") VALUES ('system:concept',NULL,'Concept','box','A reusable concept or entity in the knowledge graph.','[{"key":"definition","label":"Definition","kind":"text"}]',1,0,0);

INSERT INTO "spaces_object_types" ("id","space_id","name","icon","description","fields","system","created_at","updated_at") VALUES ('system:decision',NULL,'Decision','git-branch','A decision with explicit rationale and knowledge context.','[{"key":"decision","label":"Decision","kind":"text","required":true},{"key":"owner","label":"Owner","kind":"text"}]',1,0,0);

INSERT INTO "spaces_object_types" ("id","space_id","name","icon","description","fields","system","created_at","updated_at") VALUES ('system:evidence',NULL,'Evidence','quote','A source-grounded observation or excerpt.','[{"key":"observation","label":"Observation","kind":"text","required":true}]',1,0,0);

INSERT INTO "spaces_object_types" ("id","space_id","name","icon","description","fields","system","created_at","updated_at") VALUES ('system:experiment',NULL,'Experiment','flask-conical','A hypothesis, method, and result that can generate evidence.','[{"key":"hypothesis","label":"Hypothesis","kind":"text"},{"key":"result","label":"Result","kind":"text"}]',1,0,0);

INSERT INTO "spaces_object_types" ("id","space_id","name","icon","description","fields","system","created_at","updated_at") VALUES ('system:question',NULL,'Question','circle-help','An open or resolved research question.','[{"key":"question","label":"Question","kind":"text","required":true}]',1,0,0);

CREATE UNIQUE INDEX spaces_single_inbox ON spaces(kind) WHERE kind = 'inbox';

CREATE INDEX spaces_lifecycle ON spaces(archived_at, last_opened_at DESC);

CREATE UNIQUE INDEX spaces_object_types_name ON spaces_object_types(space_id, name COLLATE NOCASE);

CREATE INDEX spaces_nodes_space_created ON spaces_nodes(space_id, deleted_at, created_at);

CREATE INDEX spaces_nodes_object_type ON spaces_nodes(space_id, object_type_id, deleted_at);

CREATE INDEX spaces_nodes_portal_target_space ON spaces_nodes(json_extract(data, '$.targetSpaceId')) WHERE kind = 'portal' AND deleted_at IS NULL;

CREATE UNIQUE INDEX spaces_nodes_space_id ON spaces_nodes(space_id, id);

CREATE UNIQUE INDEX spaces_structure_space_id ON spaces_structure_nodes(space_id, id);

CREATE UNIQUE INDEX spaces_structure_space_id_kind ON spaces_structure_nodes(space_id, id, kind);

CREATE UNIQUE INDEX spaces_structure_active_content ON spaces_structure_nodes(node_id)
  WHERE deleted_at IS NULL AND node_id IS NOT NULL;

CREATE UNIQUE INDEX spaces_structure_active_order
  ON spaces_structure_nodes(space_id, COALESCE(parent_id, ''), order_index)
  WHERE deleted_at IS NULL;

CREATE INDEX spaces_structure_children
  ON spaces_structure_nodes(space_id, parent_id, deleted_at, order_index);

-- An FTS5 table has one key, its rowid: it supports no secondary index, so addressing a row by the
-- owner id stored beside the text makes every delete scan the whole index. These triggers mirror
-- the owner's rowid into the index and address rows by it, which is the difference between a seek
-- and a scan on each removed row — and between linear and quadratic when a Space is deleted.
CREATE TRIGGER spaces_structure_title_fts_insert AFTER INSERT ON spaces_structure_nodes
WHEN NEW.deleted_at IS NULL AND NEW.kind IN ('group','section')
BEGIN
  INSERT INTO spaces_structure_title_fts(rowid,structure_node_id,space_id,kind,title)
  VALUES(NEW.rowid,NEW.id,NEW.space_id,NEW.kind,NEW.title);
END;

-- Only the indexed columns may rewrite the index. Reordering a Space updates order_index on
-- every sibling, and an unguarded trigger reindexed each of those titles for nothing.
CREATE TRIGGER spaces_structure_title_fts_update AFTER UPDATE ON spaces_structure_nodes
WHEN OLD.title IS NOT NEW.title OR OLD.deleted_at IS NOT NEW.deleted_at
  OR OLD.kind <> NEW.kind OR OLD.space_id <> NEW.space_id
BEGIN
  DELETE FROM spaces_structure_title_fts WHERE rowid=OLD.rowid;
  INSERT INTO spaces_structure_title_fts(rowid,structure_node_id,space_id,kind,title)
  SELECT NEW.rowid,NEW.id,NEW.space_id,NEW.kind,NEW.title
  WHERE NEW.deleted_at IS NULL AND NEW.kind IN ('group','section');
END;

CREATE TRIGGER spaces_structure_title_fts_delete AFTER DELETE ON spaces_structure_nodes
BEGIN
  DELETE FROM spaces_structure_title_fts WHERE rowid=OLD.rowid;
END;

CREATE INDEX spaces_edit_receipts_space_revision
  ON spaces_edit_receipts(space_id, applied_revision);

CREATE INDEX spaces_edit_history_cursor ON spaces_edit_history(space_id, undone, id);

-- A coordinated edit stores one record per Space and replays both together.
CREATE INDEX spaces_edit_history_group ON spaces_edit_history(history_group_id, undone, id)
  WHERE history_group_id IS NOT NULL;

CREATE VIEW spaces_semantic_nodes AS
  SELECT * FROM spaces_nodes
  WHERE deleted_at IS NULL AND kind NOT IN ('credential','portal','text');

CREATE INDEX spaces_scenes_space_created ON spaces_scenes(space_id, order_index, created_at);

CREATE UNIQUE INDEX spaces_relations_unique_active
  ON spaces_relations(space_id, source_id, target_id, kind) WHERE status <> 'invalidated';

CREATE INDEX spaces_relations_space_status ON spaces_relations(space_id, status, created_at);

CREATE UNIQUE INDEX spaces_relations_space_id ON spaces_relations(space_id, id);

CREATE UNIQUE INDEX spaces_derivations_output ON spaces_knowledge_derivations(output_node_id);

CREATE INDEX spaces_derivations_space_status ON spaces_knowledge_derivations(space_id, status, created_at);

CREATE INDEX spaces_derivation_inputs_node ON spaces_knowledge_derivation_inputs(node_id, derivation_id);

CREATE UNIQUE INDEX spaces_derivation_inputs_live_identity
  ON spaces_knowledge_derivation_inputs(derivation_id, node_id, role) WHERE node_id IS NOT NULL;

CREATE INDEX spaces_provenance_target ON spaces_provenance(space_id, target_kind, target_id, status, created_at);

CREATE INDEX spaces_provenance_source ON spaces_provenance(source_node_id, status, created_at);

CREATE INDEX spaces_knowledge_events_entity ON spaces_knowledge_events(space_id, entity_kind, entity_id, created_at);

CREATE INDEX spaces_knowledge_events_time ON spaces_knowledge_events(space_id, created_at DESC);

CREATE INDEX spaces_knowledge_node ON spaces_knowledge_chunks(node_id, origin, created_at);

CREATE INDEX spaces_knowledge_space_kind ON spaces_knowledge_chunks(space_id, kind, created_at);

CREATE INDEX spaces_knowledge_revision ON spaces_knowledge_chunks(node_id, source_revision);

CREATE TRIGGER spaces_knowledge_chunks_fts_insert AFTER INSERT ON spaces_knowledge_chunks BEGIN
  INSERT INTO spaces_knowledge_fts(rowid, chunk_id, space_id, node_id, text)
  VALUES (NEW.rowid, NEW.id, NEW.space_id, NEW.node_id, NEW.text);
END;

CREATE TRIGGER spaces_knowledge_chunks_fts_update AFTER UPDATE ON spaces_knowledge_chunks
WHEN OLD.text != NEW.text BEGIN
  DELETE FROM spaces_knowledge_fts WHERE rowid = OLD.rowid;
  INSERT INTO spaces_knowledge_fts(rowid, chunk_id, space_id, node_id, text)
  VALUES (NEW.rowid, NEW.id, NEW.space_id, NEW.node_id, NEW.text);
END;

CREATE TRIGGER spaces_knowledge_chunks_fts_delete AFTER DELETE ON spaces_knowledge_chunks BEGIN
  DELETE FROM spaces_knowledge_fts WHERE rowid = OLD.rowid;
END;

CREATE TRIGGER spaces_node_chunk_remove AFTER UPDATE OF deleted_at ON spaces_nodes
WHEN new.deleted_at IS NOT NULL BEGIN
  DELETE FROM spaces_knowledge_chunks WHERE id = 'node:' || new.id;
END;

CREATE UNIQUE INDEX spaces_relations_symmetric_pair_current
  ON spaces_relations(
    space_id, kind,
    CASE WHEN source_id < target_id THEN source_id ELSE target_id END,
    CASE WHEN source_id < target_id THEN target_id ELSE source_id END
  )
  WHERE kind IN ('related', 'contradicts') AND status <> 'invalidated';

CREATE INDEX spaces_knowledge_events_entity_global
  ON spaces_knowledge_events(entity_kind, entity_id, created_at DESC, id DESC);

CREATE TRIGGER spaces_relations_same_space_insert
BEFORE INSERT ON spaces_relations BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes s JOIN spaces_nodes t
      ON t.id = NEW.target_id AND t.space_id = NEW.space_id
    WHERE s.id = NEW.source_id AND s.space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Relation endpoints must belong to relation Space') END;
END;

CREATE TRIGGER spaces_relations_same_space_update
BEFORE UPDATE OF space_id, source_id, target_id ON spaces_relations BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes s JOIN spaces_nodes t
      ON t.id = NEW.target_id AND t.space_id = NEW.space_id
    WHERE s.id = NEW.source_id AND s.space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Relation endpoints must belong to relation Space') END;
END;

CREATE TRIGGER spaces_derivations_same_space_insert
BEFORE INSERT ON spaces_knowledge_derivations BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes
    WHERE id = NEW.output_node_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Derivation output must belong to derivation Space') END;
END;

CREATE TRIGGER spaces_derivations_same_space_update
BEFORE UPDATE OF space_id, output_node_id ON spaces_knowledge_derivations BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes
    WHERE id = NEW.output_node_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Derivation output must belong to derivation Space') END;
END;

CREATE TRIGGER spaces_derivation_inputs_same_space_insert
BEFORE INSERT ON spaces_knowledge_derivation_inputs WHEN NEW.node_id IS NOT NULL BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_knowledge_derivations d JOIN spaces_nodes n
      ON n.id = NEW.node_id AND n.space_id = d.space_id
    WHERE d.id = NEW.derivation_id AND n.id <> d.output_node_id
  ) THEN RAISE(ABORT, 'Derivation input must belong to derivation Space and differ from output') END;
END;

CREATE TRIGGER spaces_derivation_inputs_same_space_update
BEFORE UPDATE OF derivation_id, node_id ON spaces_knowledge_derivation_inputs WHEN NEW.node_id IS NOT NULL BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_knowledge_derivations d JOIN spaces_nodes n
      ON n.id = NEW.node_id AND n.space_id = d.space_id
    WHERE d.id = NEW.derivation_id AND n.id <> d.output_node_id
  ) THEN RAISE(ABORT, 'Derivation input must belong to derivation Space and differ from output') END;
END;

CREATE TRIGGER spaces_chunks_same_space_insert
BEFORE INSERT ON spaces_knowledge_chunks BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes WHERE id = NEW.node_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Knowledge chunk node must belong to chunk Space') END;
END;

CREATE TRIGGER spaces_chunks_same_space_update
BEFORE UPDATE OF space_id, node_id ON spaces_knowledge_chunks BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes WHERE id = NEW.node_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Knowledge chunk node must belong to chunk Space') END;
END;

CREATE TRIGGER spaces_provenance_same_space_insert
BEFORE INSERT ON spaces_provenance BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes WHERE id = NEW.source_node_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Provenance source must belong to Provenance Space') END;
  SELECT CASE WHEN NEW.target_kind = 'node' AND NOT EXISTS (
    SELECT 1 FROM spaces_nodes WHERE id = NEW.target_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Provenance node target must belong to Provenance Space') END;
  SELECT CASE WHEN NEW.target_kind = 'relation' AND NOT EXISTS (
    SELECT 1 FROM spaces_relations WHERE id = NEW.target_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Provenance relation target must belong to Provenance Space') END;
END;

CREATE TRIGGER spaces_provenance_same_space_update
BEFORE UPDATE OF space_id, target_kind, target_id, source_node_id ON spaces_provenance BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes WHERE id = NEW.source_node_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Provenance source must belong to Provenance Space') END;
  SELECT CASE WHEN NEW.target_kind = 'node' AND NOT EXISTS (
    SELECT 1 FROM spaces_nodes WHERE id = NEW.target_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Provenance node target must belong to Provenance Space') END;
  SELECT CASE WHEN NEW.target_kind = 'relation' AND NOT EXISTS (
    SELECT 1 FROM spaces_relations WHERE id = NEW.target_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Provenance relation target must belong to Provenance Space') END;
END;

CREATE UNIQUE INDEX spaces_retrieval_invalidation_space
  ON spaces_retrieval_invalidations(space_id) WHERE node_id IS NULL;

CREATE UNIQUE INDEX spaces_retrieval_invalidation_node
  ON spaces_retrieval_invalidations(space_id, node_id) WHERE node_id IS NOT NULL;

CREATE UNIQUE INDEX spaces_retrieval_generation_active
  ON spaces_retrieval_generations(space_id) WHERE status = 'active';

CREATE UNIQUE INDEX spaces_retrieval_generation_building
  ON spaces_retrieval_generations(space_id) WHERE status = 'building';

CREATE INDEX spaces_chunk_embeddings_spec
  ON spaces_chunk_embeddings(embedding_spec_id, generation_id, updated_at);

CREATE UNIQUE INDEX spaces_enrichment_jobs_unique_pending
  ON spaces_enrichment_jobs(space_id, node_id, job_type, source_revision)
  WHERE status IN ('pending', 'running', 'awaiting_configuration');

CREATE INDEX spaces_enrichment_jobs_status ON spaces_enrichment_jobs(status, updated_at);

CREATE TRIGGER spaces_chunk_embedding_invalidate AFTER UPDATE OF text, source_revision ON spaces_knowledge_chunks
WHEN old.text <> new.text OR old.source_revision <> new.source_revision BEGIN
  DELETE FROM spaces_chunk_embeddings WHERE chunk_id = new.id;
END;

CREATE TRIGGER spaces_node_chunk_insert AFTER INSERT ON spaces_nodes
WHEN new.deleted_at IS NULL AND new.kind NOT IN ('credential','portal','text') BEGIN
  INSERT INTO spaces_knowledge_chunks(
    id, space_id, node_id, origin, kind, source_revision, text, locator, metadata, created_at, updated_at
  ) VALUES (
    'node:' || new.id, new.space_id, new.id, 'node', 'node',
    COALESCE(json_extract(new.data, '$.knowledgeRevision'), 'node:' || new.updated_at),
    substr(trim(
      COALESCE(json_extract(new.data, '$.title'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.summary'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.url'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.siteName'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.description'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.fileName'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.tags'), '')
    ), 1, 12000),
    json_object(
      'sourceRevision', COALESCE(json_extract(new.data, '$.knowledgeRevision'), 'node:' || new.updated_at),
      'location', json_object('kind', 'node')
    ),
    '{}', new.created_at, new.updated_at
  );
END;

CREATE TRIGGER spaces_node_chunk_update AFTER UPDATE OF data, space_id, deleted_at ON spaces_nodes
WHEN new.deleted_at IS NULL AND new.kind NOT IN ('credential','portal','text') BEGIN
  INSERT INTO spaces_knowledge_chunks(
    id, space_id, node_id, origin, kind, source_revision, text, locator, metadata, created_at, updated_at
  ) VALUES (
    'node:' || new.id, new.space_id, new.id, 'node', 'node',
    COALESCE(json_extract(new.data, '$.knowledgeRevision'), 'node:' || new.updated_at),
    substr(trim(
      COALESCE(json_extract(new.data, '$.title'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.summary'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.url'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.siteName'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.description'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.fileName'), '') || ' ' ||
      COALESCE(json_extract(new.data, '$.tags'), '')
    ), 1, 12000),
    json_object(
      'sourceRevision', COALESCE(json_extract(new.data, '$.knowledgeRevision'), 'node:' || new.updated_at),
      'location', json_object('kind', 'node')
    ),
    '{}', new.created_at, new.updated_at
  ) ON CONFLICT(id) DO UPDATE SET
    space_id = excluded.space_id, kind = excluded.kind, source_revision = excluded.source_revision,
    text = excluded.text, locator = excluded.locator, updated_at = excluded.updated_at;
END;

CREATE TRIGGER spaces_node_chunk_space_follow AFTER UPDATE OF space_id ON spaces_nodes
WHEN old.space_id <> new.space_id BEGIN
  UPDATE spaces_knowledge_chunks
  SET space_id = new.space_id, updated_at = MAX(updated_at, new.updated_at)
  WHERE node_id = new.id AND space_id <> new.space_id;
END;

CREATE TRIGGER spaces_chunks_revision_insert
BEFORE INSERT ON spaces_knowledge_chunks
WHEN NEW.origin <> 'node' BEGIN
  SELECT CASE
    WHEN NEW.origin = 'derived' AND NOT EXISTS (
      SELECT 1 FROM spaces_nodes n
      WHERE n.id = NEW.node_id AND n.space_id = NEW.space_id
        AND NEW.source_revision = COALESCE(json_extract(n.data, '$.knowledgeRevision'), '')
    ) THEN RAISE(ABORT, 'Derived Knowledge chunk revision must match node knowledgeRevision')
    WHEN NEW.origin IN ('source','media') AND NOT EXISTS (
      SELECT 1 FROM spaces_nodes n
      WHERE n.id = NEW.node_id AND n.space_id = NEW.space_id
        AND NEW.source_revision = COALESCE(json_extract(n.data, '$.sourceRevision'), '')
    ) THEN RAISE(ABORT, 'Source Knowledge chunk revision must match node sourceRevision')
  END;
END;

CREATE TRIGGER spaces_chunks_revision_update
BEFORE UPDATE OF space_id, node_id, origin, source_revision ON spaces_knowledge_chunks
WHEN NEW.origin <> 'node' BEGIN
  SELECT CASE
    WHEN NEW.origin = 'derived' AND NOT EXISTS (
      SELECT 1 FROM spaces_nodes n
      WHERE n.id = NEW.node_id AND n.space_id = NEW.space_id
        AND NEW.source_revision = COALESCE(json_extract(n.data, '$.knowledgeRevision'), '')
    ) THEN RAISE(ABORT, 'Derived Knowledge chunk revision must match node knowledgeRevision')
    WHEN NEW.origin IN ('source','media') AND NOT EXISTS (
      SELECT 1 FROM spaces_nodes n
      WHERE n.id = NEW.node_id AND n.space_id = NEW.space_id
        AND NEW.source_revision = COALESCE(json_extract(n.data, '$.sourceRevision'), '')
    ) THEN RAISE(ABORT, 'Source Knowledge chunk revision must match node sourceRevision')
  END;
END;

CREATE TRIGGER spaces_chunks_passage_insert
BEFORE INSERT ON spaces_knowledge_chunks BEGIN
  SELECT CASE
    WHEN length(trim(NEW.text)) = 0 THEN RAISE(ABORT, 'Knowledge passage text must not be empty')
    WHEN length(CAST(NEW.text AS BLOB)) > 49152 THEN RAISE(ABORT, 'Knowledge passage exceeds the 48 KiB UTF-8 limit')
  END;
END;

CREATE TRIGGER spaces_chunks_passage_update
BEFORE UPDATE OF text ON spaces_knowledge_chunks BEGIN
  SELECT CASE
    WHEN length(trim(NEW.text)) = 0 THEN RAISE(ABORT, 'Knowledge passage text must not be empty')
    WHEN length(CAST(NEW.text AS BLOB)) > 49152 THEN RAISE(ABORT, 'Knowledge passage exceeds the 48 KiB UTF-8 limit')
  END;
END;

CREATE VIEW spaces_current_knowledge_chunks AS
SELECT c.id, c.space_id, c.node_id, c.origin, c.kind, c.source_revision, c.text,
       c.locator, c.metadata, c.created_at, c.updated_at
FROM spaces_knowledge_chunks c
JOIN spaces_nodes n ON n.id = c.node_id AND n.space_id = c.space_id
WHERE n.deleted_at IS NULL
  AND (
    c.origin = 'node'
    OR (c.origin = 'derived' AND c.source_revision = COALESCE(json_extract(n.data, '$.knowledgeRevision'), ''))
    OR (c.origin IN ('source', 'media') AND c.source_revision = COALESCE(json_extract(n.data, '$.sourceRevision'), ''))
  );

CREATE INDEX spaces_authorization_grants_principal
  ON spaces_authorization_grants(principal_kind, principal_id, valid_until);

CREATE INDEX spaces_runs_recovery ON spaces_runs(state, updated_at);

CREATE INDEX spaces_runs_unfinished_deadline
  ON spaces_runs(created_at + CAST(json_extract(budget,'$.maxDurationSeconds') AS INTEGER) * 1000)
  WHERE completed_at IS NULL AND cancel_requested_at IS NULL;

CREATE INDEX spaces_runs_unfinished_controls
  ON spaces_runs(id)
  WHERE completed_at IS NULL;

CREATE INDEX spaces_run_attempts_execution_lease
  ON spaces_run_attempts(lease_until)
  WHERE state IN ('claimed','executing');

CREATE INDEX spaces_run_steps_ready_availability
  ON spaces_run_steps(available_at, run_id)
  WHERE state='ready';

CREATE INDEX spaces_runs_space_created ON spaces_runs(home_space_id, created_at DESC);

CREATE UNIQUE INDEX spaces_runs_active_voice_candidate
  ON spaces_runs(scope_kind)
  WHERE scope_kind='application' AND completed_at IS NULL
    AND json_extract(input_snapshot,'$.workflow')='speech_voice_candidate_v1';

CREATE UNIQUE INDEX spaces_runs_active_speech_project
  ON spaces_runs(json_extract(input_snapshot,'$.projectId'))
  WHERE kind='workflow' AND completed_at IS NULL
    AND json_extract(input_snapshot,'$.workflow')='speech_project_generation_v1';

CREATE INDEX spaces_run_steps_ready ON spaces_run_steps(state, updated_at);

CREATE INDEX spaces_run_events_run_sequence ON spaces_run_events(run_id, sequence);

CREATE INDEX spaces_external_agent_conversations_space
  ON spaces_external_agent_conversations(space_id, updated_at DESC);

CREATE INDEX spaces_external_output_staging_path
  ON spaces_external_output_staging(path);

CREATE UNIQUE INDEX spaces_external_agent_turns_active
  ON spaces_external_agent_turns(conversation_id) WHERE state='running';

CREATE TRIGGER spaces_derivation_run_publish_guard
BEFORE INSERT ON spaces_knowledge_derivations
WHEN EXISTS (
  SELECT 1 FROM spaces_runs
  WHERE id=NEW.id AND (
    cancel_requested_at IS NOT NULL OR completed_at IS NOT NULL
    OR state NOT IN ('queued','running')
  )
)
BEGIN
  SELECT RAISE(ABORT, 'run-publish-fenced');
END;

CREATE INDEX spaces_conversations_space
  ON spaces_conversations(space_id, updated_at DESC);

CREATE INDEX spaces_conversation_messages_branch
  ON spaces_conversation_messages(conversation_id, branch_id, created_at, id);

CREATE TABLE spaces_voice_profiles (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
  source_kind TEXT NOT NULL CHECK (source_kind IN ('preset','reference','designed')),
  active_revision_id TEXT REFERENCES spaces_voice_revisions(id) ON DELETE SET NULL,
  archived_at INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE spaces_voice_revisions (
  id TEXT PRIMARY KEY NOT NULL,
  profile_id TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision > 0),
  engine_id TEXT NOT NULL CHECK (length(trim(engine_id)) > 0),
  model_spec_id TEXT NOT NULL CHECK (length(trim(model_spec_id)) > 0),
  language TEXT NOT NULL CHECK (length(trim(language)) > 0),
  preset_voice_id TEXT,
  reference_asset_id TEXT,
  reference_source_revision TEXT,
  reference_text TEXT,
  design_description TEXT,
  synthesis_config TEXT NOT NULL CHECK (json_valid(synthesis_config) AND json_type(synthesis_config)='object'),
  authorization TEXT NOT NULL CHECK (json_valid(authorization) AND json_type(authorization)='object'),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (profile_id) REFERENCES spaces_voice_profiles(id) ON DELETE CASCADE,
  UNIQUE (profile_id, revision),
  CHECK (
    (preset_voice_id IS NOT NULL AND reference_asset_id IS NULL AND reference_source_revision IS NULL
      AND reference_text IS NULL AND design_description IS NULL)
    OR (preset_voice_id IS NULL AND reference_asset_id IS NOT NULL AND reference_source_revision IS NOT NULL)
  )
);

CREATE TRIGGER spaces_voice_revision_immutable BEFORE UPDATE ON spaces_voice_revisions
BEGIN
  SELECT RAISE(ABORT, 'VoiceRevision is immutable; create a new revision');
END;

CREATE TABLE spaces_voice_candidates (
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
      OR (source_kind='reference' AND design_description IS NULL
          AND input_source_revision IS NOT NULL AND input_byte_length IS NOT NULL)),
  CHECK ((status='ready' AND selected_voice_profile_id IS NULL AND selected_voice_revision_id IS NULL AND selected_at IS NULL)
      OR (status='selected' AND auditioned_at IS NOT NULL AND selected_voice_profile_id IS NOT NULL
          AND selected_voice_revision_id IS NOT NULL AND selected_at IS NOT NULL)
)
);

CREATE TABLE spaces_speech_projects (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  source_node_id TEXT NOT NULL,
  project_type TEXT NOT NULL CHECK (project_type IN ('short_dubbing','audiobook','video_dubbing')),
  title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 200),
  active_revision_id TEXT REFERENCES spaces_speech_project_revisions(id) ON DELETE SET NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (source_node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE
);

CREATE TRIGGER spaces_speech_project_source_insert
BEFORE INSERT ON spaces_speech_projects BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes n WHERE n.id=NEW.source_node_id AND n.space_id=NEW.space_id
      AND n.deleted_at IS NULL AND (
        (NEW.project_type='short_dubbing' AND n.kind IN ('note','text','markdown')) OR
        (NEW.project_type='audiobook' AND n.kind IN ('note','text','markdown','file')) OR
        (NEW.project_type='video_dubbing' AND n.kind='video')
      )
  ) THEN RAISE(ABORT, 'Speech project requires a matching source node in its Space') END;
END;

CREATE TRIGGER spaces_speech_project_source_immutable
BEFORE UPDATE OF space_id,source_node_id,project_type ON spaces_speech_projects
WHEN NEW.space_id<>OLD.space_id OR NEW.source_node_id<>OLD.source_node_id OR NEW.project_type<>OLD.project_type
BEGIN
  SELECT RAISE(ABORT, 'Speech project source identity is immutable');
END;

CREATE TRIGGER spaces_speech_source_space_immutable
BEFORE UPDATE OF space_id,kind ON spaces_nodes
WHEN (NEW.space_id<>OLD.space_id OR NEW.kind<>OLD.kind)
  AND EXISTS (SELECT 1 FROM spaces_speech_projects WHERE source_node_id=OLD.id)
BEGIN
  SELECT RAISE(ABORT, 'A speech source node cannot change its Space or type while it owns productions');
END;

CREATE TABLE spaces_speech_project_revisions (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision > 0),
  source_revision TEXT NOT NULL CHECK (length(trim(source_revision)) > 0),
  pronunciation_lexicon TEXT NOT NULL CHECK (json_valid(pronunciation_lexicon) AND json_type(pronunciation_lexicon)='object'),
  output_config TEXT NOT NULL CHECK (json_valid(output_config) AND json_type(output_config)='object'),
  output_asset_path TEXT,
  output_sha256 TEXT CHECK (output_sha256 IS NULL OR length(output_sha256)=64),
  output_sample_rate INTEGER CHECK (output_sample_rate IS NULL OR output_sample_rate > 0),
  output_sample_count INTEGER CHECK (output_sample_count IS NULL OR output_sample_count >= 0),
  output_published_at INTEGER,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (project_id) REFERENCES spaces_speech_projects(id) ON DELETE CASCADE,
  UNIQUE (project_id, revision),
  CHECK (
    (output_asset_path IS NULL AND output_sha256 IS NULL AND output_sample_rate IS NULL
      AND output_sample_count IS NULL AND output_published_at IS NULL)
    OR
    (output_asset_path IS NOT NULL AND output_sha256 IS NOT NULL AND output_sample_rate IS NOT NULL
      AND output_sample_count IS NOT NULL AND output_published_at IS NOT NULL)
  )
);

CREATE TABLE spaces_speech_segments (
  id TEXT PRIMARY KEY NOT NULL,
  project_revision_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
  chapter_key TEXT,
  speaker_key TEXT,
  text TEXT NOT NULL CHECK (length(trim(text)) > 0),
  normalized_text TEXT NOT NULL CHECK (length(trim(normalized_text)) > 0),
  delivery_instruction TEXT CHECK (delivery_instruction IS NULL OR length(trim(delivery_instruction)) BETWEEN 1 AND 500),
  voice_revision_id TEXT NOT NULL,
  language TEXT NOT NULL CHECK (length(trim(language)) > 0),
  target_start_ms INTEGER CHECK (target_start_ms IS NULL OR target_start_ms >= 0),
  target_end_ms INTEGER CHECK (target_end_ms IS NULL OR target_end_ms > target_start_ms),
  source_segment_id TEXT,
  source_segment_revision INTEGER CHECK (source_segment_revision IS NULL OR source_segment_revision > 0),
  source_text TEXT CHECK (source_text IS NULL OR length(trim(source_text)) > 0),
  state TEXT NOT NULL CHECK (state IN ('ready','rendering','rendered','failed')),
  state_reason TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (project_revision_id) REFERENCES spaces_speech_project_revisions(id) ON DELETE CASCADE,
  FOREIGN KEY (voice_revision_id) REFERENCES spaces_voice_revisions(id) ON DELETE RESTRICT,
  UNIQUE (project_revision_id, ordinal),
  CHECK (
    (source_segment_id IS NULL AND source_segment_revision IS NULL AND source_text IS NULL)
    OR
    (length(trim(source_segment_id)) BETWEEN 1 AND 240
      AND source_segment_revision IS NOT NULL AND source_text IS NOT NULL)
  )
);

CREATE TABLE spaces_speech_renders (
  id TEXT PRIMARY KEY NOT NULL,
  segment_id TEXT NOT NULL,
  attempt_epoch INTEGER NOT NULL CHECK (attempt_epoch >= 0),
  run_id TEXT,
  run_step_id TEXT,
  run_attempt_id TEXT,
  run_execution_epoch INTEGER CHECK (run_execution_epoch IS NULL OR run_execution_epoch >= 0),
  input_digest TEXT NOT NULL CHECK (length(input_digest)=64),
  model_spec_id TEXT NOT NULL,
  engine_version TEXT NOT NULL,
  seed INTEGER,
  synthesis_config TEXT NOT NULL CHECK (json_valid(synthesis_config) AND json_type(synthesis_config)='object'),
  state TEXT NOT NULL CHECK (state IN ('staged','published','rejected','failed','cancelled')),
  asset_path TEXT,
  output_sha256 TEXT CHECK (output_sha256 IS NULL OR length(output_sha256)=64),
  sample_rate INTEGER CHECK (sample_rate IS NULL OR sample_rate > 0),
  sample_count INTEGER CHECK (sample_count IS NULL OR sample_count >= 0),
  created_at INTEGER NOT NULL,
  published_at INTEGER,
  FOREIGN KEY (segment_id) REFERENCES spaces_speech_segments(id) ON DELETE CASCADE,
  UNIQUE (segment_id, attempt_epoch),
  CHECK (
    (run_id IS NULL AND run_step_id IS NULL AND run_attempt_id IS NULL AND run_execution_epoch IS NULL)
    OR
    (run_id IS NOT NULL AND run_step_id IS NOT NULL AND run_attempt_id IS NOT NULL AND run_execution_epoch IS NOT NULL)
  ),
  CHECK (
    (state='staged' AND asset_path IS NOT NULL AND output_sha256 IS NULL
      AND sample_rate IS NULL AND sample_count IS NULL AND published_at IS NULL)
    OR
    (state='published' AND asset_path IS NOT NULL AND output_sha256 IS NOT NULL
      AND sample_rate IS NOT NULL AND sample_count IS NOT NULL AND published_at IS NOT NULL)
    OR
    (state IN ('rejected','failed','cancelled') AND asset_path IS NULL AND output_sha256 IS NULL
      AND sample_rate IS NULL AND sample_count IS NULL AND published_at IS NULL)
  )
);

CREATE TABLE spaces_speech_asset_publications (
  id TEXT PRIMARY KEY NOT NULL,
  scope_kind TEXT NOT NULL CHECK (scope_kind IN ('space','application')),
  space_id TEXT,
  owner_kind TEXT NOT NULL CHECK (owner_kind IN ('segment_render','chapter_checkpoint','project_output','voice_reference','voice_candidate','voice_candidate_input','source_separation')),
  owner_id TEXT NOT NULL CHECK (length(trim(owner_id)) > 0),
  asset_path TEXT NOT NULL UNIQUE,
  part_path TEXT,
  sha256 TEXT NOT NULL CHECK (length(sha256)=64),
  -- run_id, run_step_id and run_attempt_id record which execution produced the file. They are
  -- provenance identities rather than references: a Run can be pruned without unpublishing an
  -- asset, and the all-or-none CHECK below rules out clearing one of them on delete.
  run_id TEXT,
  run_step_id TEXT,
  run_attempt_id TEXT,
  run_execution_epoch INTEGER CHECK (run_execution_epoch IS NULL OR run_execution_epoch >= 0),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  CHECK ((scope_kind='application' AND space_id IS NULL
      AND owner_kind IN ('voice_reference','voice_candidate','voice_candidate_input'))
    OR (scope_kind='space' AND space_id IS NOT NULL AND length(trim(space_id))>0
      AND owner_kind NOT IN ('voice_reference','voice_candidate','voice_candidate_input'))),
  CHECK (
    (run_id IS NULL AND run_step_id IS NULL AND run_attempt_id IS NULL AND run_execution_epoch IS NULL)
    OR
    (run_id IS NOT NULL AND run_step_id IS NOT NULL AND run_attempt_id IS NOT NULL AND run_execution_epoch IS NOT NULL)
  )
);

CREATE TABLE spaces_speech_chapter_checkpoints (
  id TEXT PRIMARY KEY NOT NULL,
  project_revision_id TEXT NOT NULL,
  chapter_key TEXT NOT NULL,
  input_digest TEXT NOT NULL CHECK (length(input_digest)=64),
  asset_path TEXT NOT NULL,
  sha256 TEXT NOT NULL CHECK (length(sha256)=64),
  sample_rate INTEGER NOT NULL CHECK (sample_rate > 0),
  sample_count INTEGER NOT NULL CHECK (sample_count >= 0),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (project_revision_id) REFERENCES spaces_speech_project_revisions(id) ON DELETE CASCADE,
  UNIQUE (project_revision_id, chapter_key, input_digest)
);

CREATE TABLE spaces_speech_loudness_checkpoints (
  id TEXT PRIMARY KEY NOT NULL,
  project_revision_id TEXT NOT NULL,
  input_digest TEXT NOT NULL CHECK (length(input_digest)=64),
  measurement TEXT NOT NULL CHECK (json_valid(measurement) AND json_type(measurement)='object'),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (project_revision_id) REFERENCES spaces_speech_project_revisions(id) ON DELETE CASCADE,
  UNIQUE (project_revision_id, input_digest)
);

CREATE TABLE spaces_speech_source_separations (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  run_id TEXT NOT NULL UNIQUE,
  source_node_id TEXT NOT NULL,
  source_asset_path TEXT NOT NULL,
  source_revision TEXT NOT NULL CHECK (length(source_revision)=71 AND substr(source_revision,1,7)='sha256:'),
  source_duration_ms INTEGER NOT NULL CHECK (source_duration_ms > 0 AND source_duration_ms <= 600000),
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

CREATE INDEX spaces_voice_profiles_library ON spaces_voice_profiles(archived_at, updated_at DESC);
CREATE INDEX spaces_voice_revisions_profile ON spaces_voice_revisions(profile_id, revision DESC);
CREATE INDEX spaces_voice_candidates_library ON spaces_voice_candidates(status, created_at DESC);
CREATE INDEX spaces_speech_projects_space ON spaces_speech_projects(space_id, updated_at DESC);
CREATE INDEX spaces_speech_asset_publications_space ON spaces_speech_asset_publications(space_id, created_at);
CREATE INDEX spaces_speech_segments_revision ON spaces_speech_segments(project_revision_id, ordinal);
CREATE INDEX spaces_speech_renders_segment ON spaces_speech_renders(segment_id, attempt_epoch DESC);
CREATE INDEX spaces_speech_source_separations_space ON spaces_speech_source_separations(space_id, created_at DESC);
CREATE INDEX spaces_reserved_outputs_run ON spaces_reserved_outputs(run_id);
CREATE UNIQUE INDEX spaces_speech_renders_run_attempt
  ON spaces_speech_renders(run_attempt_id) WHERE run_attempt_id IS NOT NULL;
CREATE UNIQUE INDEX spaces_speech_asset_publications_attempt_asset
  ON spaces_speech_asset_publications(run_attempt_id,asset_path) WHERE run_attempt_id IS NOT NULL;

CREATE TRIGGER spaces_voice_revision_source_guard
BEFORE INSERT ON spaces_voice_revisions BEGIN
  SELECT CASE
    WHEN NOT EXISTS (
      SELECT 1 FROM spaces_voice_profiles p WHERE p.id=NEW.profile_id AND (
        (p.source_kind='preset' AND NEW.preset_voice_id IS NOT NULL)
        OR (p.source_kind='reference' AND NEW.reference_asset_id IS NOT NULL)
        OR (p.source_kind='designed' AND NEW.reference_asset_id IS NOT NULL AND NEW.design_description IS NOT NULL)
      )
    ) THEN RAISE(ABORT, 'Voice revision source does not match its profile')
  END;
END;

CREATE TRIGGER spaces_voice_profile_active_revision_guard
BEFORE UPDATE OF active_revision_id ON spaces_voice_profiles
WHEN NEW.active_revision_id IS NOT NULL BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_voice_revisions r WHERE r.id=NEW.active_revision_id AND r.profile_id=NEW.id
  ) THEN RAISE(ABORT, 'Active voice revision does not belong to its profile') END;
END;

CREATE TRIGGER spaces_speech_project_active_revision_guard
BEFORE UPDATE OF active_revision_id ON spaces_speech_projects
WHEN NEW.active_revision_id IS NOT NULL BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_speech_project_revisions r WHERE r.id=NEW.active_revision_id AND r.project_id=NEW.id
  ) THEN RAISE(ABORT, 'Active speech revision does not belong to its project') END;
END;

-- Every foreign key needs an index on its child columns: without one SQLite scans the whole
-- child table for each parent row it deletes or nulls out, so a Space or node removal degrades
-- with the size of the library. `spaces/db/tests` enforces this for every foreign key.
-- Trashing a node must prove no live portal or object still points at it. Only those two kinds
-- can, so they are indexed apart from the rest of the Space.
CREATE INDEX spaces_nodes_reference_holders ON spaces_nodes(space_id, kind)
  WHERE deleted_at IS NULL AND kind IN ('portal','object');

-- Endpoint first so the index serves the foreign key, Space second so invalidating the
-- relations of a deleted node stays a pair of index lookups.
CREATE INDEX spaces_relations_source ON spaces_relations(source_id, space_id);
CREATE INDEX spaces_relations_target ON spaces_relations(target_id, space_id);
CREATE INDEX spaces_nodes_object_type_reference ON spaces_nodes(object_type_id);
CREATE INDEX spaces_structure_nodes_node ON spaces_structure_nodes(space_id, node_id);
-- No index for spaces_structure_layouts: its primary key (space_id, structure_node_id) already
-- seeks every row of its foreign key, which carries only `kind` beyond that unique pair. A third
-- index would duplicate the table key for every card and earn nothing; SQLite chooses the primary
-- key for the cascade either way.
CREATE INDEX spaces_enrichment_jobs_node ON spaces_enrichment_jobs(node_id);
CREATE INDEX spaces_enrichment_jobs_space ON spaces_enrichment_jobs(space_id);
CREATE INDEX spaces_chunk_embeddings_chunk ON spaces_chunk_embeddings(chunk_id);
CREATE INDEX spaces_retrieval_generations_space ON spaces_retrieval_generations(space_id);
-- The drain reads whole-Space tombstones before node tombstones, in insertion order.
CREATE INDEX spaces_retrieval_invalidations_drain
  ON spaces_retrieval_invalidations(space_id, node_id, id);
CREATE INDEX spaces_reserved_outputs_space ON spaces_reserved_outputs(space_id);
CREATE INDEX spaces_action_invocations_space ON spaces_action_invocations(space_id);
CREATE INDEX spaces_agent_artifact_sessions_space ON spaces_agent_artifact_sessions(space_id);
CREATE INDEX spaces_authorization_grants_space ON spaces_authorization_grants(home_space_id);
CREATE INDEX spaces_runs_parent ON spaces_runs(parent_run_id);
CREATE INDEX spaces_runs_retry_of ON spaces_runs(retry_of_run_id);
CREATE INDEX spaces_runs_fork_of ON spaces_runs(fork_of_run_id);
CREATE INDEX spaces_run_attempts_run ON spaces_run_attempts(run_id);
CREATE INDEX spaces_conversation_mcp_interactions_run ON spaces_conversation_mcp_interactions(run_id);
CREATE INDEX spaces_conversation_messages_parent ON spaces_conversation_messages(parent_message_id);
CREATE INDEX spaces_conversation_tool_operations_task ON spaces_conversation_tool_operations(task_id);
CREATE INDEX spaces_external_agent_conversations_conversation ON spaces_external_agent_conversations(conversation_id);
CREATE INDEX spaces_external_agent_turns_conversation ON spaces_external_agent_turns(conversation_id);
CREATE INDEX spaces_external_agent_turns_message ON spaces_external_agent_turns(message_id);
CREATE INDEX spaces_builtin_agent_execution_segments_conversation ON spaces_builtin_agent_execution_segments(conversation_id);
CREATE INDEX spaces_builtin_agent_tasks_conversation ON spaces_builtin_agent_tasks(conversation_id);
CREATE INDEX spaces_builtin_agent_tasks_segment ON spaces_builtin_agent_tasks(segment_id);
CREATE INDEX spaces_builtin_agent_tasks_grant ON spaces_builtin_agent_tasks(grant_id);
CREATE INDEX spaces_builtin_agent_model_calls_task ON spaces_builtin_agent_model_calls(task_id);
CREATE INDEX spaces_builtin_agent_tool_admissions_task ON spaces_builtin_agent_tool_admissions(task_id);
CREATE INDEX spaces_speech_projects_source ON spaces_speech_projects(source_node_id);
CREATE INDEX spaces_speech_source_separations_source ON spaces_speech_source_separations(source_node_id);
CREATE INDEX spaces_speech_segments_voice_revision ON spaces_speech_segments(voice_revision_id);
CREATE INDEX spaces_voice_profiles_active_revision ON spaces_voice_profiles(active_revision_id);
CREATE INDEX spaces_speech_projects_active_revision ON spaces_speech_projects(active_revision_id);
