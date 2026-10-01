import type { DocumentCoverage } from "../../model/nodes/schema/document-coverage-fields";
import { knowledgeLocationLabelFromLocation } from "../knowledge/knowledge-location-label";

type DocumentResource = DocumentCoverage["resources"][number];
type DocumentOccurrence = DocumentCoverage["occurrences"][number];

export interface DocumentResourceProjection {
  resource: DocumentResource;
  occurrences: readonly DocumentOccurrence[];
  locationLabel: string;
  sourceKind: "embedded" | "rendered";
}

export function projectDocumentResources(report: DocumentCoverage): readonly DocumentResourceProjection[] {
  const occurrencesByResource = new Map<string, DocumentOccurrence[]>();
  for (const occurrence of report.occurrences) {
    if (occurrence.resourceId === null) continue;
    const occurrences = occurrencesByResource.get(occurrence.resourceId) ?? [];
    occurrences.push(occurrence);
    occurrencesByResource.set(occurrence.resourceId, occurrences);
  }
  return report.resources.map((resource) => {
    const occurrences = occurrencesByResource.get(resource.id)!;
    const primary = occurrences[0]!;
    return {
      resource,
      occurrences,
      locationLabel: knowledgeLocationLabelFromLocation(primary.location),
      sourceKind: report.parser === "pdf-inspector" ? "rendered" : "embedded",
    };
  });
}
