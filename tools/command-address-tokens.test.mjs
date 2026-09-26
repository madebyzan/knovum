import assert from "node:assert/strict";
import test from "node:test";

import { matched, tokenPattern } from "./command-address-tokens.mjs";

test("complete IPC literals are recognized without treating template prefixes as commands", () => {
  const pattern = tokenPattern("plugin:[a-z0-9_-]+\\|[a-z0-9_]+");
  const source = [
    'invoke("plugin:spaces|allocate_document_as_pdf", {});',
    "const stable = 'plugin:spaces|export_document_as_docx';",
    "const staticTemplate = `plugin:spaces|load_space_session_projection`;",
    "const dynamic = `plugin:spaces|export_document_as_${format}`;",
    "const mismatched = 'plugin:spaces|not_a_command\";",
  ].join("\n");
  assert.deepEqual(
    [...matched(source, pattern)],
    [
      "plugin:spaces|allocate_document_as_pdf",
      "plugin:spaces|export_document_as_docx",
      "plugin:spaces|load_space_session_projection",
    ],
  );
});

test("disabled namespace patterns produce no command candidates", () => {
  assert.equal(tokenPattern(""), null);
  assert.deepEqual([...matched("anything", null)], []);
});
