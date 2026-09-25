DROP TABLE spaces_external_tool_bindings;

-- Historical grounded answers retain their verified source snapshots in one conversation store.
INSERT INTO spaces_assistant_conversations(id,space_id,title,mode,created_at,updated_at)
SELECT id,space_id,title,'ask',created_at,updated_at FROM spaces_ai_threads;

INSERT INTO spaces_assistant_messages(id,conversation_id,branch_id,role,content,created_at)
SELECT id,thread_id,thread_id,role,
       json_object('parts',json_array(json_object('type','text','text',content)),
                   'citations',json(citations)),created_at
FROM spaces_ai_messages;

DROP TABLE spaces_ai_messages;
DROP TABLE spaces_ai_threads;

-- A desktop Agent session is an execution binding of the same visible conversation.
INSERT INTO spaces_assistant_conversations(id,space_id,title,mode,created_at,updated_at)
SELECT id,space_id,title,'ask',created_at,updated_at
FROM spaces_external_agent_conversations;

INSERT INTO spaces_assistant_messages(id,conversation_id,branch_id,role,content,created_at)
SELECT id,conversation_id,conversation_id,role,
       json_object('parts',json_array(json_object('type','text','text',content))),created_at
FROM spaces_external_agent_messages;

DROP TABLE spaces_external_agent_messages;
ALTER TABLE spaces_external_agent_conversations
  ADD COLUMN assistant_conversation_id TEXT REFERENCES spaces_assistant_conversations(id) ON DELETE CASCADE;
ALTER TABLE spaces_external_agent_conversations ADD COLUMN branch_id TEXT;
UPDATE spaces_external_agent_conversations
  SET assistant_conversation_id=id,branch_id=id;
ALTER TABLE spaces_external_agent_turns ADD COLUMN assistant_message_id TEXT
  REFERENCES spaces_assistant_messages(id) ON DELETE SET NULL;

ALTER TABLE spaces_assistant_conversations ADD COLUMN active_branch_id TEXT;
ALTER TABLE spaces_assistant_tasks ADD COLUMN error TEXT;
UPDATE spaces_assistant_conversations SET active_branch_id=COALESCE(
  (SELECT branch_id FROM spaces_assistant_messages
   WHERE conversation_id=spaces_assistant_conversations.id
   ORDER BY created_at DESC,id DESC LIMIT 1),id);

CREATE TABLE spaces_captured_workflow_versions (
  workflow_id TEXT NOT NULL,
  version INTEGER NOT NULL CHECK (version > 0),
  definition TEXT NOT NULL CHECK (json_valid(definition)),
  created_at INTEGER NOT NULL,
  PRIMARY KEY (workflow_id, version),
  FOREIGN KEY (workflow_id) REFERENCES spaces_workflow_templates(id) ON DELETE CASCADE
);
INSERT INTO spaces_captured_workflow_versions(workflow_id,version,definition,created_at)
SELECT id,json_extract(definition,'$.version'),definition,created_at
FROM spaces_workflow_templates WHERE json_extract(definition,'$.schemaVersion')=3;
