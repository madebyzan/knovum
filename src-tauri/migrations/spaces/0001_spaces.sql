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
  result_json TEXT NOT NULL CHECK (json_valid(result_json)),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_deleted_structure_batches (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  root_structure_node_ids TEXT NOT NULL CHECK (json_valid(root_structure_node_ids) AND json_type(root_structure_node_ids) = 'array'),
  deleted_at INTEGER NOT NULL,
  restored_at INTEGER,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_deleted_structure_nodes (
  batch_id TEXT NOT NULL,
  structure_node_id TEXT NOT NULL,
  data TEXT NOT NULL CHECK (json_valid(data) AND json_type(data) = 'object'),
  PRIMARY KEY (batch_id, structure_node_id),
  FOREIGN KEY (batch_id) REFERENCES spaces_deleted_structure_batches(id) ON DELETE CASCADE
);

CREATE TABLE spaces_deleted_structure_layouts (
  batch_id TEXT NOT NULL,
  structure_node_id TEXT NOT NULL,
  data TEXT NOT NULL CHECK (json_valid(data) AND json_type(data) = 'object'),
  PRIMARY KEY (batch_id, structure_node_id),
  FOREIGN KEY (batch_id) REFERENCES spaces_deleted_structure_batches(id) ON DELETE CASCADE
);

CREATE TABLE spaces_deleted_structure_relations (
  batch_id TEXT NOT NULL,
  relation_id TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('active','contested','invalidated')),
  status_before_invalidation TEXT CHECK (status_before_invalidation IN ('active','contested')),
  color TEXT CHECK (color IN ('blue','green','amber','red','purple')),
  PRIMARY KEY (batch_id, relation_id),
  FOREIGN KEY (batch_id) REFERENCES spaces_deleted_structure_batches(id) ON DELETE CASCADE
);

CREATE TABLE spaces_edit_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  space_id TEXT NOT NULL,
  before_snapshot TEXT NOT NULL CHECK (json_valid(before_snapshot)),
  after_snapshot TEXT NOT NULL CHECK (json_valid(after_snapshot)),
  before_node_ids TEXT NOT NULL CHECK (json_valid(before_node_ids) AND json_type(before_node_ids) = 'array'),
  after_node_ids TEXT NOT NULL CHECK (json_valid(after_node_ids) AND json_type(after_node_ids) = 'array'),
  before_relations TEXT NOT NULL CHECK (json_valid(before_relations) AND json_type(before_relations) = 'array'),
  after_relations TEXT NOT NULL CHECK (json_valid(after_relations) AND json_type(after_relations) = 'array'),
  deletion_batch_id TEXT,
  deletion_batch_restored_after INTEGER CHECK (deletion_batch_restored_after IN (0,1)),
  history_group_id TEXT,
  expected_revision INTEGER NOT NULL CHECK (expected_revision >= 0),
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
  actor_kind TEXT NOT NULL DEFAULT 'human' CHECK (actor_kind IN ('human', 'assistant', 'agent', 'system')),
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
  action_kind TEXT NOT NULL CHECK (action_kind IN ('remove-image-background', 'remove-image-object', 'split-image-layers', 'upscale-image', 'generate-image-angle', 'generate-visualization', 'generate-image', 'add-video-subtitles', 'render-video-edit', 'draft-document', 'draft-script', 'summarize-meeting', 'draft-social-post', 'export-document-as-docx', 'export-document-as-pdf', 'export-document-as-txt', 'render-presentation', 'generate-video')),
  action_version INTEGER NOT NULL CHECK (action_version = 1),
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

CREATE TABLE spaces_grounded_documents (
  derivation_id TEXT PRIMARY KEY NOT NULL,
  citations TEXT NOT NULL CHECK (json_valid(citations) AND json_type(citations) = 'array'),
  execution_source TEXT NOT NULL CHECK (json_valid(execution_source)),
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE
);

CREATE TABLE spaces_generated_images (
  derivation_id TEXT PRIMARY KEY NOT NULL,
  intent TEXT NOT NULL,
  style TEXT NOT NULL,
  aspect_ratio TEXT NOT NULL CHECK (aspect_ratio IN ('1:1','3:2','2:3','16:9','9:16')),
  execution_source TEXT NOT NULL CHECK (json_valid(execution_source)),
  revised_prompt TEXT,
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE
);

CREATE TABLE spaces_generated_videos (
  derivation_id TEXT PRIMARY KEY NOT NULL,
  prompt TEXT NOT NULL,
  aspect_ratio TEXT NOT NULL CHECK (aspect_ratio IN ('1:1','16:9','9:16')),
  duration_seconds INTEGER NOT NULL CHECK (duration_seconds BETWEEN 5 AND 15),
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
  actor_kind TEXT NOT NULL CHECK (actor_kind IN ('human', 'assistant', 'agent', 'system')),
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
  actor_kind TEXT NOT NULL CHECK (actor_kind IN ('human', 'assistant', 'agent', 'system')),
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

CREATE TABLE spaces_chunk_embeddings (
  chunk_id TEXT PRIMARY KEY NOT NULL,
  source_revision TEXT NOT NULL,
  model TEXT NOT NULL,
  dimensions INTEGER NOT NULL CHECK (dimensions > 0),
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (chunk_id) REFERENCES spaces_knowledge_chunks(id) ON DELETE CASCADE
);

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
  job_type TEXT NOT NULL CHECK (job_type IN ('analyze_visual', 'analyze_media', 'transcribe_media', 'parse_document', 'analyze_document')),
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

CREATE TABLE spaces_document_visual_checkpoints (
  job_id TEXT NOT NULL REFERENCES spaces_enrichment_jobs(id) ON DELETE CASCADE,
  occurrence_id TEXT NOT NULL,
  occurrence TEXT NOT NULL CHECK (json_valid(occurrence)),
  resource TEXT CHECK (resource IS NULL OR json_valid(resource)),
  PRIMARY KEY (job_id, occurrence_id)
);

CREATE TABLE spaces_authorization_grants (
  id TEXT PRIMARY KEY NOT NULL,
  principal_kind TEXT NOT NULL CHECK (principal_kind IN ('user','assistant','external_agent','automation')),
  principal_id TEXT NOT NULL,
  home_space_id TEXT NOT NULL,
  read_space_ids TEXT NOT NULL CHECK (json_valid(read_space_ids) AND json_type(read_space_ids) = 'array'),
  write_space_ids TEXT NOT NULL CHECK (json_valid(write_space_ids) AND json_type(write_space_ids) = 'array'),
  action_ids TEXT NOT NULL CHECK (json_valid(action_ids) AND json_type(action_ids) = 'array'),
  valid_until INTEGER NOT NULL,
  revoked_at INTEGER,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (home_space_id) REFERENCES spaces(id) ON DELETE CASCADE
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
  outcome_kind TEXT NOT NULL CHECK (outcome_kind IN ('query_result','committed_receipt','accepted_run')),
  outcome TEXT NOT NULL CHECK (json_valid(outcome)),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  UNIQUE (principal_kind, principal_id, space_id, action_id, action_version, idempotency_key)
);

CREATE TABLE spaces_runs (
  id TEXT PRIMARY KEY NOT NULL,
  home_space_id TEXT NOT NULL,
  parent_run_id TEXT,
  retry_of_run_id TEXT,
  fork_of_run_id TEXT,
  principal_kind TEXT NOT NULL,
  principal_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('action','workflow','assistant','external_agent','automation')),
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
  FOREIGN KEY (fork_of_run_id) REFERENCES spaces_runs(id) ON DELETE SET NULL
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
  result TEXT CHECK (result IS NULL OR json_valid(result)),
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
  updated_at INTEGER NOT NULL, assistant_conversation_id TEXT REFERENCES spaces_assistant_conversations(id) ON DELETE CASCADE, branch_id TEXT,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_external_agent_turns (
  id TEXT PRIMARY KEY NOT NULL,
  conversation_id TEXT NOT NULL,
  state TEXT NOT NULL CHECK (state IN ('running','completed','failed','cancelled')),
  error TEXT,
  created_at INTEGER NOT NULL,
  completed_at INTEGER, assistant_message_id TEXT
  REFERENCES spaces_assistant_messages(id) ON DELETE SET NULL,
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

CREATE TABLE spaces_workflow_templates (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  definition TEXT NOT NULL CHECK (json_valid(definition)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE spaces_assistant_conversations (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  title TEXT NOT NULL,
  mode TEXT NOT NULL CHECK (mode IN ('ask','plan','execute')),
  archived_at INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL, active_branch_id TEXT,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_assistant_messages (
  id TEXT PRIMARY KEY NOT NULL,
  conversation_id TEXT NOT NULL,
  branch_id TEXT NOT NULL,
  parent_message_id TEXT,
  role TEXT NOT NULL CHECK (role IN ('user','assistant','tool')),
  content TEXT NOT NULL CHECK (json_valid(content)),
  created_at INTEGER NOT NULL,
  FOREIGN KEY (conversation_id) REFERENCES spaces_assistant_conversations(id) ON DELETE CASCADE,
  FOREIGN KEY (parent_message_id) REFERENCES spaces_assistant_messages(id) ON DELETE SET NULL
);

CREATE TABLE spaces_assistant_context_segments (
  id TEXT PRIMARY KEY NOT NULL,
  conversation_id TEXT NOT NULL,
  branch_id TEXT NOT NULL,
  source_digest TEXT NOT NULL CHECK (length(source_digest) = 64),
  first_message_id TEXT NOT NULL,
  last_message_id TEXT NOT NULL,
  message_count INTEGER NOT NULL CHECK (message_count > 0),
  summary TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  UNIQUE (conversation_id, branch_id, source_digest),
  FOREIGN KEY (conversation_id) REFERENCES spaces_assistant_conversations(id) ON DELETE CASCADE
);

CREATE TABLE spaces_assistant_tasks (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL UNIQUE,
  conversation_id TEXT NOT NULL,
  branch_id TEXT NOT NULL,
  grant_id TEXT NOT NULL,
  model_binding TEXT NOT NULL CHECK (json_valid(model_binding)),
  catalog_digest TEXT NOT NULL CHECK (length(catalog_digest) = 64),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL, error TEXT,
  FOREIGN KEY (run_id) REFERENCES spaces_runs(id) ON DELETE CASCADE,
  FOREIGN KEY (conversation_id) REFERENCES spaces_assistant_conversations(id) ON DELETE CASCADE,
  FOREIGN KEY (grant_id) REFERENCES spaces_authorization_grants(id) ON DELETE RESTRICT
);

CREATE TABLE spaces_assistant_model_turns (
  id TEXT PRIMARY KEY NOT NULL,
  task_id TEXT NOT NULL,
  sequence INTEGER NOT NULL CHECK (sequence > 0),
  request TEXT NOT NULL CHECK (json_valid(request)),
  response TEXT CHECK (response IS NULL OR json_valid(response)),
  state TEXT NOT NULL CHECK (state IN ('requesting','completed','failed','outcome_unknown')),
  created_at INTEGER NOT NULL,
  completed_at INTEGER,
  FOREIGN KEY (task_id) REFERENCES spaces_assistant_tasks(id) ON DELETE CASCADE,
  UNIQUE (task_id, sequence)
);

CREATE TABLE spaces_assistant_tool_calls (
  id TEXT PRIMARY KEY NOT NULL,
  task_id TEXT NOT NULL,
  model_turn_id TEXT NOT NULL,
  provider_call_id TEXT NOT NULL,
  wire_name TEXT NOT NULL,
  action_id TEXT NOT NULL,
  invocation_id TEXT NOT NULL,
  result TEXT CHECK (result IS NULL OR json_valid(result)),
  created_at INTEGER NOT NULL,
  completed_at INTEGER,
  FOREIGN KEY (task_id) REFERENCES spaces_assistant_tasks(id) ON DELETE CASCADE,
  FOREIGN KEY (model_turn_id) REFERENCES spaces_assistant_model_turns(id) ON DELETE CASCADE,
  UNIQUE (task_id, provider_call_id),
  UNIQUE (invocation_id)
);

CREATE TABLE spaces_assistant_checkpoints (
  task_id TEXT PRIMARY KEY NOT NULL,
  envelope TEXT NOT NULL CHECK (json_valid(envelope)),
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (task_id) REFERENCES spaces_assistant_tasks(id) ON DELETE CASCADE
);

CREATE TABLE spaces_captured_workflow_versions (
  workflow_id TEXT NOT NULL,
  version INTEGER NOT NULL CHECK (version > 0),
  definition TEXT NOT NULL CHECK (json_valid(definition)),
  created_at INTEGER NOT NULL,
  PRIMARY KEY (workflow_id, version),
  FOREIGN KEY (workflow_id) REFERENCES spaces_workflow_templates(id) ON DELETE CASCADE
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

CREATE TRIGGER spaces_structure_title_fts_insert AFTER INSERT ON spaces_structure_nodes
WHEN NEW.deleted_at IS NULL AND NEW.kind IN ('group','section')
BEGIN
  INSERT INTO spaces_structure_title_fts(structure_node_id,space_id,kind,title)
  VALUES(NEW.id,NEW.space_id,NEW.kind,NEW.title);
END;

CREATE TRIGGER spaces_structure_title_fts_update AFTER UPDATE ON spaces_structure_nodes
BEGIN
  DELETE FROM spaces_structure_title_fts WHERE structure_node_id=OLD.id;
  INSERT INTO spaces_structure_title_fts(structure_node_id,space_id,kind,title)
  SELECT NEW.id,NEW.space_id,NEW.kind,NEW.title
  WHERE NEW.deleted_at IS NULL AND NEW.kind IN ('group','section');
END;

CREATE TRIGGER spaces_structure_title_fts_delete AFTER DELETE ON spaces_structure_nodes
BEGIN
  DELETE FROM spaces_structure_title_fts WHERE structure_node_id=OLD.id;
END;

CREATE INDEX spaces_edit_receipts_space_revision
  ON spaces_edit_receipts(space_id, applied_revision);

CREATE INDEX spaces_deleted_structure_batches_space
  ON spaces_deleted_structure_batches(space_id, deleted_at DESC);

CREATE INDEX spaces_edit_history_cursor ON spaces_edit_history(space_id, undone, id);

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
  INSERT INTO spaces_knowledge_fts(chunk_id, space_id, node_id, text)
  VALUES (NEW.id, NEW.space_id, NEW.node_id, NEW.text);
END;

CREATE TRIGGER spaces_knowledge_chunks_fts_update AFTER UPDATE ON spaces_knowledge_chunks
WHEN OLD.text != NEW.text BEGIN
  DELETE FROM spaces_knowledge_fts WHERE chunk_id = OLD.id;
  INSERT INTO spaces_knowledge_fts(chunk_id, space_id, node_id, text)
  VALUES (NEW.id, NEW.space_id, NEW.node_id, NEW.text);
END;

CREATE TRIGGER spaces_knowledge_chunks_fts_delete AFTER DELETE ON spaces_knowledge_chunks BEGIN
  DELETE FROM spaces_knowledge_fts WHERE chunk_id = OLD.id;
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

CREATE INDEX spaces_chunk_embeddings_model ON spaces_chunk_embeddings(model, updated_at);

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

CREATE INDEX spaces_runs_space_created ON spaces_runs(home_space_id, created_at DESC);

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

CREATE INDEX spaces_assistant_conversations_space
  ON spaces_assistant_conversations(space_id, updated_at DESC);

CREATE INDEX spaces_assistant_messages_branch
  ON spaces_assistant_messages(conversation_id, branch_id, created_at, id);
