-- Pre-release contract reset: every Canvas node now declares whether its content
-- policy or persisted frame owns its dimensions. Canvas state is a projection and
-- is rebuilt from the surviving Space nodes on next open.
DELETE FROM spaces_canvas_asset_references
WHERE space_id IN (
  SELECT state.space_id
  FROM spaces_canvas_state AS state
  WHERE EXISTS (
    SELECT 1
    FROM json_each(state.document, '$.nodes') AS node
    WHERE json_type(node.value, '$.sizeAuthority') IS NULL
  )
);

DELETE FROM spaces_canvas_state
WHERE EXISTS (
  SELECT 1
  FROM json_each(spaces_canvas_state.document, '$.nodes') AS node
  WHERE json_type(node.value, '$.sizeAuthority') IS NULL
);
