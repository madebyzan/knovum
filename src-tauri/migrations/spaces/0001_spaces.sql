-- Space aggregate, canvas objects, scenes, and knowledge graph relations.
CREATE TABLE spaces (
  id TEXT PRIMARY KEY NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('inbox', 'space')),
  title TEXT NOT NULL,
  archived_at INTEGER,
  favorite INTEGER NOT NULL DEFAULT 0 CHECK (favorite IN (0, 1)),
  thumbnail_data_url TEXT,
  last_opened_at INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE UNIQUE INDEX spaces_single_inbox ON spaces(kind) WHERE kind = 'inbox';

CREATE INDEX spaces_lifecycle ON spaces(archived_at, last_opened_at DESC);

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

CREATE UNIQUE INDEX spaces_object_types_name ON spaces_object_types(space_id, name COLLATE NOCASE);

CREATE TABLE spaces_nodes (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (
    kind IN (
      'note', 'text', 'website', 'markdown', 'image', 'video', 'audio', 'subtitle', 'file', 'code',
      'folder', 'archive', 'credential', 'section', 'object', 'portal'
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

CREATE INDEX spaces_nodes_space_created ON spaces_nodes(space_id, deleted_at, created_at);

CREATE INDEX spaces_nodes_object_type ON spaces_nodes(space_id, object_type_id, deleted_at);

CREATE TABLE spaces_canvas_state (
  space_id TEXT PRIMARY KEY NOT NULL,
  document TEXT NOT NULL CHECK (json_valid(document)),
  revision INTEGER NOT NULL CHECK (revision > 0),
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_canvas_asset_references (
  space_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  path TEXT NOT NULL CHECK (length(path) > 0),
  PRIMARY KEY (space_id, node_id),
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE
);

CREATE TRIGGER spaces_canvas_asset_reference_same_space_insert
BEFORE INSERT ON spaces_canvas_asset_references BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes WHERE id = NEW.node_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Canvas asset reference node must belong to Canvas Space') END;
END;

CREATE TRIGGER spaces_canvas_asset_reference_same_space_update
BEFORE UPDATE OF space_id, node_id ON spaces_canvas_asset_references BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_nodes WHERE id = NEW.node_id AND space_id = NEW.space_id
  ) THEN RAISE(ABORT, 'Canvas asset reference node must belong to Canvas Space') END;
END;

CREATE VIEW spaces_semantic_nodes AS
  SELECT * FROM spaces_nodes
  WHERE deleted_at IS NULL AND kind NOT IN ('credential','section','portal');

CREATE TABLE spaces_scenes (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  title TEXT NOT NULL,
  viewport_x REAL NOT NULL,
  viewport_y REAL NOT NULL,
  viewport_zoom REAL NOT NULL,
  thumbnail_data_url TEXT NOT NULL,
  order_index INTEGER NOT NULL DEFAULT 0,
  focus_node_ids TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(focus_node_ids)),
  relation_kinds TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(relation_kinds)),
  relation_statuses TEXT NOT NULL DEFAULT '["active","contested"]' CHECK (json_valid(relation_statuses)),
  query_filter TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(query_filter)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE INDEX spaces_scenes_space_created ON spaces_scenes(space_id, order_index, created_at);

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

CREATE UNIQUE INDEX spaces_relations_unique_active
  ON spaces_relations(space_id, source_id, target_id, kind) WHERE status <> 'invalidated';

CREATE INDEX spaces_relations_space_status ON spaces_relations(space_id, status, created_at);

-- Knowledge derivations, workflow runs, and generated artifact metadata.
CREATE TABLE spaces_knowledge_derivations (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  output_node_id TEXT NOT NULL,
  action_kind TEXT NOT NULL CHECK (action_kind IN ('image-background-removal', 'knowledge-visualization', 'video-subtitle-extraction', 'video-subtitle-burn-in', 'grounded-document-draft', 'markdown-docx-export', 'markdown-pdf-export', 'knowledge-presentation', 'knowledge-video')),
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

CREATE UNIQUE INDEX spaces_derivations_output ON spaces_knowledge_derivations(output_node_id);
CREATE INDEX spaces_derivations_space_status ON spaces_knowledge_derivations(space_id, status, created_at);

CREATE TABLE spaces_knowledge_derivation_inputs (
  derivation_id TEXT NOT NULL,
  position INTEGER NOT NULL CHECK (position >= 0),
  node_id TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('primary', 'source', 'context', 'style-reference')),
  source_revision TEXT NOT NULL,
  PRIMARY KEY (derivation_id, position),
  UNIQUE (derivation_id, node_id, role),
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE
);

CREATE INDEX spaces_derivation_inputs_node ON spaces_knowledge_derivation_inputs(node_id, derivation_id);

CREATE TABLE spaces_creation_workflow_runs (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  workflow_id TEXT NOT NULL,
  workflow_version INTEGER NOT NULL CHECK (workflow_version > 0),
  status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),
  error TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  completed_at INTEGER,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  CHECK ((status = 'completed' AND completed_at IS NOT NULL AND error = '') OR
         (status = 'failed' AND completed_at IS NOT NULL AND length(error) > 0) OR
         (status = 'cancelled' AND completed_at IS NOT NULL AND error = '') OR
         (status = 'running' AND completed_at IS NULL AND error = ''))
);

CREATE INDEX spaces_workflow_runs_space_status
  ON spaces_creation_workflow_runs(space_id, status, created_at);

CREATE TABLE spaces_creation_workflow_steps (
  run_id TEXT NOT NULL,
  step_id TEXT NOT NULL,
  action TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'succeeded', 'failed', 'cancelled')),
  output_node_ids TEXT NOT NULL DEFAULT '[]'
    CHECK (json_valid(output_node_ids) AND json_type(output_node_ids) = 'array'),
  error TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  completed_at INTEGER,
  PRIMARY KEY (run_id, step_id),
  FOREIGN KEY (run_id) REFERENCES spaces_creation_workflow_runs(id) ON DELETE CASCADE,
  CHECK ((status = 'succeeded' AND completed_at IS NOT NULL AND error = '') OR
         (status = 'failed' AND completed_at IS NOT NULL AND length(error) > 0) OR
         (status = 'cancelled' AND completed_at IS NOT NULL AND error = '') OR
         (status IN ('pending', 'running') AND completed_at IS NULL AND error = ''))
);

CREATE TABLE spaces_grounded_documents (
  derivation_id TEXT PRIMARY KEY NOT NULL,
  citations TEXT NOT NULL CHECK (json_valid(citations) AND json_type(citations) = 'array'),
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE
);

CREATE TABLE spaces_generated_images (
  derivation_id TEXT PRIMARY KEY NOT NULL,
  intent TEXT NOT NULL,
  style TEXT NOT NULL,
  aspect_ratio TEXT NOT NULL CHECK (aspect_ratio IN ('1:1','3:2','2:3','16:9','9:16')),
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  revised_prompt TEXT,
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE
);

CREATE TABLE spaces_generated_videos (
  derivation_id TEXT PRIMARY KEY NOT NULL,
  prompt TEXT NOT NULL,
  aspect_ratio TEXT NOT NULL CHECK (aspect_ratio IN ('1:1','16:9','9:16')),
  duration_seconds INTEGER NOT NULL CHECK (duration_seconds BETWEEN 5 AND 15),
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  operation_id TEXT NOT NULL,
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE
);

-- Provenance, domain events, canonical passages, and full-text search.
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

CREATE INDEX spaces_provenance_target ON spaces_provenance(space_id, target_kind, target_id, status, created_at);

CREATE INDEX spaces_provenance_source ON spaces_provenance(source_node_id, status, created_at);

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

CREATE INDEX spaces_knowledge_events_entity ON spaces_knowledge_events(space_id, entity_kind, entity_id, created_at);

CREATE INDEX spaces_knowledge_events_time ON spaces_knowledge_events(space_id, created_at DESC);

CREATE TABLE spaces_knowledge_chunks (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  origin TEXT NOT NULL CHECK (origin IN ('node', 'source', 'media', 'derived')),
  kind TEXT NOT NULL CHECK (
    kind IN ('node', 'text', 'heading', 'page', 'slide', 'sheet', 'table', 'transcript', 'chapter', 'code', 'image_description', 'video_first_frame')
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

CREATE INDEX spaces_knowledge_node ON spaces_knowledge_chunks(node_id, origin, created_at);

CREATE INDEX spaces_knowledge_space_kind ON spaces_knowledge_chunks(space_id, kind, created_at);

CREATE INDEX spaces_knowledge_revision ON spaces_knowledge_chunks(node_id, source_revision);

CREATE VIRTUAL TABLE spaces_knowledge_fts USING fts5(
  chunk_id UNINDEXED, space_id UNINDEXED, node_id UNINDEXED, text, tokenize = 'trigram'
);

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

-- Space-scoped assistant conversations.
CREATE TABLE spaces_ai_threads (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  scope TEXT NOT NULL CHECK (scope IN ('selection', 'section', 'space', 'all-spaces')),
  scope_node_ids TEXT NOT NULL CHECK (json_valid(scope_node_ids)),
  title TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE INDEX spaces_ai_threads_space ON spaces_ai_threads(space_id, updated_at DESC);

CREATE TABLE spaces_ai_messages (
  id TEXT PRIMARY KEY NOT NULL,
  thread_id TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
  content TEXT NOT NULL,
  citations TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(citations)),
  provider TEXT,
  model TEXT,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (thread_id) REFERENCES spaces_ai_threads(id) ON DELETE CASCADE
);

CREATE INDEX spaces_ai_messages_thread ON spaces_ai_messages(thread_id, created_at);

-- Cross-table domain invariants enforced by SQLite.
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
BEFORE INSERT ON spaces_knowledge_derivation_inputs BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_knowledge_derivations d JOIN spaces_nodes n
      ON n.id = NEW.node_id AND n.space_id = d.space_id
    WHERE d.id = NEW.derivation_id AND n.id <> d.output_node_id
  ) THEN RAISE(ABORT, 'Derivation input must belong to derivation Space and differ from output') END;
END;

CREATE TRIGGER spaces_derivation_inputs_same_space_update
BEFORE UPDATE OF derivation_id, node_id ON spaces_knowledge_derivation_inputs BEGIN
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

-- Rebuildable semantic projections and enrichment work scheduling.
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

CREATE UNIQUE INDEX spaces_retrieval_invalidation_space
  ON spaces_retrieval_invalidations(space_id) WHERE node_id IS NULL;

CREATE UNIQUE INDEX spaces_retrieval_invalidation_node
  ON spaces_retrieval_invalidations(space_id, node_id) WHERE node_id IS NOT NULL;

CREATE INDEX spaces_chunk_embeddings_model ON spaces_chunk_embeddings(model, updated_at);

CREATE TABLE spaces_enrichment_jobs (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  job_type TEXT NOT NULL CHECK (job_type IN ('embed_text', 'analyze_image', 'transcribe_media', 'analyze_video_first_frame', 'embed_media_transcript')),
  status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'succeeded', 'failed', 'cancelled')),
  source_revision TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
  error TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX spaces_enrichment_jobs_unique_pending
  ON spaces_enrichment_jobs(space_id, node_id, job_type, source_revision)
  WHERE status IN ('pending', 'running');

CREATE INDEX spaces_enrichment_jobs_status ON spaces_enrichment_jobs(status, updated_at);

CREATE TRIGGER spaces_chunk_embedding_invalidate AFTER UPDATE OF text, source_revision ON spaces_knowledge_chunks
WHEN old.text <> new.text OR old.source_revision <> new.source_revision BEGIN
  DELETE FROM spaces_chunk_embeddings WHERE chunk_id = new.id;
END;

CREATE TRIGGER spaces_node_chunk_insert AFTER INSERT ON spaces_nodes
WHEN new.deleted_at IS NULL AND new.kind NOT IN ('section','credential','portal') BEGIN
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
WHEN new.deleted_at IS NULL AND new.kind NOT IN ('section','credential','portal') BEGIN
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

-- Read models and canonical system-owned object types.
CREATE INDEX spaces_ai_messages_thread_id_created_at
ON spaces_ai_messages(thread_id, created_at, id);

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

INSERT INTO spaces_object_types(id, space_id, name, icon, description, fields, system, created_at, updated_at) VALUES
  ('system:claim', NULL, 'Claim', 'lightbulb', 'An atomic assertion that can be supported, challenged, or superseded.', '[{"key":"statement","label":"Statement","kind":"text","required":true}]', 1, 0, 0),
  ('system:concept', NULL, 'Concept', 'box', 'A reusable concept or entity in the knowledge graph.', '[{"key":"definition","label":"Definition","kind":"text"}]', 1, 0, 0),
  ('system:decision', NULL, 'Decision', 'git-branch', 'A decision with explicit rationale and knowledge context.', '[{"key":"decision","label":"Decision","kind":"text","required":true},{"key":"owner","label":"Owner","kind":"text"}]', 1, 0, 0),
  ('system:evidence', NULL, 'Evidence', 'quote', 'A source-grounded observation or excerpt.', '[{"key":"observation","label":"Observation","kind":"text","required":true}]', 1, 0, 0),
  ('system:experiment', NULL, 'Experiment', 'flask-conical', 'A hypothesis, method, and result that can generate evidence.', '[{"key":"hypothesis","label":"Hypothesis","kind":"text"},{"key":"result","label":"Result","kind":"text"}]', 1, 0, 0),
  ('system:question', NULL, 'Question', 'circle-help', 'An open or resolved research question.', '[{"key":"question","label":"Question","kind":"text","required":true}]', 1, 0, 0);
