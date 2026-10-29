-- Knowledge Proposals and answer verification. Both are local working state: proposals wait for a
-- person to accept them into the Knowledge graph, verifications record which sentences of an answer
-- its sources state. Neither is part of a Space package; accepted knowledge travels as ordinary
-- knowledge, and an imported Space verifies its answers again on demand.

CREATE TABLE spaces_knowledge_proposals (
  id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  origin_kind TEXT NOT NULL CHECK (origin_kind IN ('answer')),
  origin TEXT NOT NULL CHECK (json_valid(origin)),
  rationale TEXT NOT NULL DEFAULT '' CHECK (length(rationale) <= 2000),
  changes TEXT NOT NULL CHECK (json_valid(changes) AND json_type(changes) = 'array'),
  basis TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(basis) AND json_type(basis) = 'array'),
  fingerprint TEXT NOT NULL,
  state TEXT NOT NULL CHECK (state IN ('pending', 'accepted', 'dismissed')),
  resolved_ids TEXT CHECK (resolved_ids IS NULL OR json_valid(resolved_ids)),
  proposer_kind TEXT NOT NULL CHECK (proposer_kind IN ('human', 'builtin_agent', 'external_agent', 'system')),
  proposer_id TEXT,
  created_at INTEGER NOT NULL,
  decided_at INTEGER,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  CHECK ((state = 'pending') = (decided_at IS NULL)),
  CHECK ((state = 'accepted') = (resolved_ids IS NOT NULL))
);
-- One proposal per meaning: a suggestion already accepted or dismissed is not proposed again.
CREATE UNIQUE INDEX spaces_knowledge_proposals_fingerprint
  ON spaces_knowledge_proposals(space_id, fingerprint);
CREATE INDEX spaces_knowledge_proposals_by_state
  ON spaces_knowledge_proposals(space_id, state, created_at);

-- The existing nodes a proposal names or quotes, for "proposals about this node" and erasure.
CREATE TABLE spaces_knowledge_proposal_targets (
  proposal_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  PRIMARY KEY (proposal_id, node_id),
  FOREIGN KEY (proposal_id) REFERENCES spaces_knowledge_proposals(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES spaces_nodes(id) ON DELETE CASCADE
);
CREATE INDEX spaces_knowledge_proposal_targets_by_node
  ON spaces_knowledge_proposal_targets(node_id);

CREATE TABLE spaces_answer_verifications (
  message_id TEXT PRIMARY KEY NOT NULL,
  space_id TEXT NOT NULL,
  -- The answer's Run: its citations are the passages the answer read.
  answer_run_id TEXT NOT NULL,
  verification_run_id TEXT,
  state TEXT NOT NULL CHECK (state IN ('pending', 'running', 'completed', 'failed', 'skipped')),
  skip TEXT CHECK (skip IN ('no-sources', 'no-statements', 'no-target')),
  statements TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(statements) AND json_type(statements) = 'array'),
  verdicts TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(verdicts) AND json_type(verdicts) = 'array'),
  corrections TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(corrections) AND json_type(corrections) = 'array'),
  truncated INTEGER NOT NULL DEFAULT 0 CHECK (truncated IN (0, 1)),
  error TEXT,
  created_at INTEGER NOT NULL,
  completed_at INTEGER,
  FOREIGN KEY (message_id) REFERENCES spaces_conversation_messages(id) ON DELETE CASCADE,
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  FOREIGN KEY (verification_run_id) REFERENCES spaces_runs(id) ON DELETE SET NULL,
  CHECK ((state = 'skipped') = (skip IS NOT NULL)),
  CHECK ((state IN ('completed', 'failed', 'skipped')) = (completed_at IS NOT NULL))
);
CREATE INDEX spaces_answer_verifications_by_space
  ON spaces_answer_verifications(space_id);
CREATE INDEX spaces_answer_verifications_by_run
  ON spaces_answer_verifications(verification_run_id);
