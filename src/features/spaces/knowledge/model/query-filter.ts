import { isUndefined, omitBy } from "es-toolkit";

import type { SpaceSavedViewFilter } from "./saved-view-filter";

export function snapshotSpaceQueryFilter(filter: SpaceSavedViewFilter): SpaceSavedViewFilter {
  return omitBy(
    {
      text: filter.text,
      tags: filter.tags ? [...filter.tags] : undefined,
      objectTypeIds: filter.objectTypeIds ? [...filter.objectTypeIds] : undefined,
      statuses: filter.statuses ? [...filter.statuses] : undefined,
      relationKinds: filter.relationKinds ? [...filter.relationKinds] : undefined,
      relationStatuses: filter.relationStatuses ? [...filter.relationStatuses] : undefined,
    },
    isUndefined,
  ) as SpaceSavedViewFilter;
}
