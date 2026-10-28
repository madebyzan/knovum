-- A node attached to a message is a reference to the canvas node: the conversation draws it from the
-- node as it is, as it already drew the nodes a reply added. Messages stopped keeping a picture of
-- one, and the picture an image attachment carried leaves with it. An uploaded file keeps its own:
-- nothing else holds that file.
UPDATE spaces_conversation_messages
SET content = json_set(
  content,
  '$.attachments',
  (
    SELECT json_group_array(json(attachment))
    FROM (
      SELECT CASE
        WHEN json_extract(entry.value, '$.source') = 'node'
          THEN json_remove(entry.value, '$.thumbnail')
        ELSE entry.value
      END AS attachment
      FROM json_each(spaces_conversation_messages.content, '$.attachments') AS entry
      ORDER BY entry.key
    )
  )
)
WHERE EXISTS (
  SELECT 1
  FROM json_each(spaces_conversation_messages.content, '$.attachments') AS entry
  WHERE json_extract(entry.value, '$.source') = 'node'
    AND json_type(entry.value, '$.thumbnail') IS NOT NULL
);
