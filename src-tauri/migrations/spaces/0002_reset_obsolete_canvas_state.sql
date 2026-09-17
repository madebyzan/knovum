-- Pre-release schema reset: connector records now live exclusively in spaces_relations.
-- Canvas state is a projection and is rebuilt from the surviving Space nodes on next open.
DELETE FROM spaces_canvas_asset_references
WHERE space_id IN (
  SELECT space_id
  FROM spaces_canvas_state
  WHERE json_type(document, '$.connectors') IS NOT NULL
);

DELETE FROM spaces_canvas_state
WHERE json_type(document, '$.connectors') IS NOT NULL;
