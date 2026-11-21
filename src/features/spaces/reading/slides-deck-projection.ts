import { convertFileSrc } from "@tauri-apps/api/core";

import type { KnowledgeChunk } from "../knowledge/model/schema/persisted-chunk";
import type { SpaceFilePayload } from "../model/nodes/node-payload";

interface PresentationSlideImage {
  id: string;
  src: string;
  width: number;
  height: number;
}

interface PresentationSlideTextBlock {
  chunkIndex: number;
  text: string;
}

export interface PresentationSlideView {
  slideNumber: number;
  images: readonly PresentationSlideImage[];
  textBlocks: readonly PresentationSlideTextBlock[];
  ocrPassages: readonly string[];
}

interface SlideAccumulator {
  images: PresentationSlideImage[];
  seenImageIds: Set<string>;
  textBlocks: PresentationSlideTextBlock[];
  ocrPassages: string[];
}

function ensureSlide(bySlide: Map<number, SlideAccumulator>, slideNumber: number): SlideAccumulator {
  let entry = bySlide.get(slideNumber);
  if (!entry) {
    entry = { images: [], seenImageIds: new Set(), textBlocks: [], ocrPassages: [] };
    bySlide.set(slideNumber, entry);
  }
  return entry;
}

function collectSlideVisuals(
  bySlide: Map<number, SlideAccumulator>,
  payload: SpaceFilePayload,
  recognizedText: (resourceId: string) => string | undefined,
) {
  const coverage = payload.documentCoverage;
  if (!coverage) return;
  const resourcesById = new Map(coverage.resources.map((resource) => [resource.id, resource]));
  for (const occurrence of coverage.occurrences) {
    if (occurrence.location.kind !== "slides") continue;
    const slide = ensureSlide(bySlide, occurrence.location.slide);
    if (!occurrence.resourceId) continue;
    const resource = resourcesById.get(occurrence.resourceId);
    if (resource && !slide.seenImageIds.has(resource.id)) {
      slide.seenImageIds.add(resource.id);
      slide.images.push({
        id: resource.id,
        src: convertFileSrc(resource.path),
        width: resource.width,
        height: resource.height,
      });
    }
    const ocr = recognizedText(occurrence.resourceId)?.trim();
    if (ocr && !slide.ocrPassages.includes(ocr)) slide.ocrPassages.push(ocr);
  }
}

export function projectPresentationSlides(
  payload: SpaceFilePayload,
  chunks: readonly KnowledgeChunk[],
  recognizedText: (resourceId: string) => string | undefined,
): readonly PresentationSlideView[] {
  const bySlide = new Map<number, SlideAccumulator>();
  collectSlideVisuals(bySlide, payload, recognizedText);

  for (const [chunkIndex, chunk] of chunks.entries()) {
    if (chunk.kind === "node" || chunk.locator.location.kind !== "slides") continue;
    const slide = ensureSlide(bySlide, chunk.locator.location.slide);
    const trimmed = chunk.text.trim();
    if (trimmed) slide.textBlocks.push({ chunkIndex, text: trimmed });
  }

  return [...bySlide.entries()]
    .toSorted(([left], [right]) => left - right)
    .map(([slideNumber, entry]) => ({
      slideNumber,
      images: entry.images,
      textBlocks: entry.textBlocks,
      ocrPassages: entry.ocrPassages,
    }));
}
