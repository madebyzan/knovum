-- Preserve derivation history independently of the lifetime of its source nodes.
-- Existing inputs receive a snapshot of their source at migration time.
DROP TRIGGER spaces_derivation_inputs_same_space_insert;
DROP TRIGGER spaces_derivation_inputs_same_space_update;

CREATE TABLE spaces_knowledge_derivation_inputs_next (
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
  PRIMARY KEY (derivation_id, position),
  UNIQUE (derivation_id, source_identity, role),
  FOREIGN KEY (derivation_id) REFERENCES spaces_knowledge_derivations(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES spaces_nodes(id) ON DELETE SET NULL
);

INSERT INTO spaces_knowledge_derivation_inputs_next (
  derivation_id, position, node_id, source_identity, source_snapshot, role, source_revision
)
SELECT i.derivation_id, i.position,
  CASE WHEN n.space_id = d.space_id THEN i.node_id ELSE NULL END,
  i.node_id,
  json_object(
    'title', json_extract(n.data, '$.title'),
    'kind', n.kind,
    'originalUrl', CASE WHEN n.kind = 'website' THEN json_extract(n.data, '$.url')
      ELSE json_extract(n.data, '$.origin.url') END,
    'platform', json_extract(n.data, '$.origin.provider')
  ),
  i.role, i.source_revision
FROM spaces_knowledge_derivation_inputs i
LEFT JOIN spaces_nodes n ON n.id = i.node_id
LEFT JOIN spaces_knowledge_derivations d ON d.id = i.derivation_id;

DROP TABLE spaces_knowledge_derivation_inputs;
ALTER TABLE spaces_knowledge_derivation_inputs_next RENAME TO spaces_knowledge_derivation_inputs;

CREATE INDEX spaces_derivation_inputs_node
  ON spaces_knowledge_derivation_inputs(node_id, derivation_id);
CREATE UNIQUE INDEX spaces_derivation_inputs_live_identity
  ON spaces_knowledge_derivation_inputs(derivation_id, node_id, role) WHERE node_id IS NOT NULL;

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
