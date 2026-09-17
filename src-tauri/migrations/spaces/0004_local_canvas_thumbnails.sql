-- Preview images are disposable derived data. Keep all Spaces and Saved Views,
-- but reset their obsolete Data URLs rather than adding a dual-format reader.
ALTER TABLE spaces RENAME COLUMN thumbnail_data_url TO thumbnail_path;
UPDATE spaces SET thumbnail_path = NULL;

CREATE TABLE spaces_scenes_local (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  title TEXT NOT NULL,
  viewport_x REAL NOT NULL,
  viewport_y REAL NOT NULL,
  viewport_zoom REAL NOT NULL,
  thumbnail_path TEXT,
  order_index INTEGER NOT NULL DEFAULT 0,
  focus_node_ids TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(focus_node_ids)),
  relation_kinds TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(relation_kinds)),
  relation_statuses TEXT NOT NULL DEFAULT '["active","contested"]' CHECK (json_valid(relation_statuses)),
  query_filter TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(query_filter)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);
INSERT INTO spaces_scenes_local (
  id, space_id, title, viewport_x, viewport_y, viewport_zoom, order_index,
  focus_node_ids, relation_kinds, relation_statuses, query_filter, created_at, updated_at
)
SELECT id, space_id, title, viewport_x, viewport_y, viewport_zoom, order_index,
       focus_node_ids, relation_kinds, relation_statuses, query_filter, created_at, updated_at
FROM spaces_scenes;
DROP TABLE spaces_scenes;
ALTER TABLE spaces_scenes_local RENAME TO spaces_scenes;
CREATE INDEX spaces_scenes_space_created ON spaces_scenes(space_id, order_index, created_at);
