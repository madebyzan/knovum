import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import type { McpResourceContent } from "../../../shared/contracts/generated/McpResourceContent";
import type { McpResourceSelection } from "../../../shared/contracts/generated/McpResourceSelection";
import type { PreparedMcpResourceImport } from "../contracts/generated/PreparedMcpResourceImport";
import { useSpaceStore } from "../session/space-store";
import { spaceImportContext } from "./import-context";
import type { SpaceImportSessionLabels } from "./import-contract";
import { importSpaceSource } from "./import-session";

interface McpResourceImportInput {
  connection: McpConnection;
  selection: McpResourceSelection;
  content: McpResourceContent;
  contentIndex: number;
  labels: SpaceImportSessionLabels;
}
export async function importMcpResource(input: McpResourceImportInput) {
  const store = useSpaceStore();
  const context = spaceImportContext();
  const { connection, selection, content, contentIndex, labels } = input;
  const spaceId = store.activeSpaceId;
  if (!spaceId) throw new Error("Space is required for a resource import");
  const position = store.nextInsertionPoint();
  const staged = await invokeCommand<PreparedMcpResourceImport>("plugin:spaces|stage_mcp_resource_import", {
    connectionId: connection.id,
    revision: connection.revision,
    catalogDigest: content.catalogDigest,
    selection,
    contentDigest: content.digest,
    contentIndex,
  });
  return importSpaceSource({
    context,
    spaceId,
    position,
    labels,
    source: {
      externalSource: staged.externalSource,
      prepare: async () => staged.prepared,
      dispose: () =>
        invokeCommand("plugin:spaces|discard_mcp_resource_import", { stagingId: staged.stagingId }),
    },
  });
}
