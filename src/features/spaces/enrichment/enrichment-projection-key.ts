import type { InjectionKey, Ref } from "vue";

import type { EnrichmentJobProjection } from "../contracts/generated/EnrichmentJobProjection";

export const enrichmentProjectionKey: InjectionKey<Ref<EnrichmentJobProjection[]>> =
  Symbol("enrichment-jobs");
