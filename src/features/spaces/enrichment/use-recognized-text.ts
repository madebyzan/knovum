import { queryOptions, useQuery } from "@tanstack/vue-query";
import { computed, type Ref } from "vue";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RecognizedTextReading } from "../contracts/generated/RecognizedTextReading";

/**
 * What local recognition read from a node's pictures, one reading per picture. The text lives in the
 * node's passages; `revision` is the node's `textRecognition` record, so a new reading is fetched
 * exactly when recognition publishes one and never before it has.
 */
export function useRecognizedText(
  spaceId: Readonly<Ref<string | null | undefined>>,
  nodeId: Readonly<Ref<string>>,
  revision: Readonly<Ref<string | undefined>>,
) {
  const query = useQuery(
    computed(() =>
      queryOptions({
        queryKey: ["spaces", "recognized-text", spaceId.value, nodeId.value, revision.value] as const,
        queryFn: () =>
          invokeCommand<RecognizedTextReading[]>("plugin:spaces|list_recognized_text", {
            request: { spaceId: spaceId.value, nodeId: nodeId.value },
          }),
        enabled: Boolean(spaceId.value && revision.value),
        staleTime: Infinity,
      }),
    ),
  );
  /** The text read from one picture: a document resource by its id, the shown picture by `null`. */
  return (resourceId: string | null) =>
    query.data.value?.find((reading) => reading.resourceId === resourceId)?.text;
}
