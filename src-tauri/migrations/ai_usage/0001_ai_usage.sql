CREATE TABLE ai_token_usage_events (
  id TEXT PRIMARY KEY NOT NULL,
  occurred_at INTEGER NOT NULL CHECK (occurred_at >= 0),
  local_date TEXT NOT NULL CHECK (
    length(local_date) = 10
    AND local_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
  ),
  purpose TEXT NOT NULL CHECK (
    purpose IN ('assistant', 'editor', 'knowledge', 'knowledge_indexing', 'translation', 'visual_analysis')
  ),
  provider TEXT NOT NULL CHECK (length(provider) > 0),
  model TEXT NOT NULL CHECK (length(model) > 0),
  input_tokens INTEGER NOT NULL CHECK (input_tokens >= 0),
  output_tokens INTEGER NOT NULL CHECK (output_tokens >= 0),
  total_tokens INTEGER NOT NULL CHECK (
    total_tokens >= input_tokens + output_tokens
  )
);

CREATE INDEX ai_token_usage_events_local_date
  ON ai_token_usage_events(local_date, occurred_at);

CREATE INDEX ai_token_usage_events_purpose
  ON ai_token_usage_events(purpose, local_date);
