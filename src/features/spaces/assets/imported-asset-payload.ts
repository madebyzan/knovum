import { isUndefined, omitBy } from "es-toolkit";

import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { ImportedSpaceAsset } from "./imported-asset";
import { extractImportedMarkdownDocument } from "./markdown-document";

export function importedSpaceNodePayload(asset: ImportedSpaceAsset): SpaceNodePayload {
  if (asset.kind === "markdown") return importedMarkdownPayload(asset);
  const common = importedAssetCommon(asset);
  if (asset.kind === "image" || asset.kind === "video") return importedVisualPayload(asset, common);
  if (asset.kind === "audio") return { ...common, kind: "audio" } as SpaceNodePayload;
  if (asset.kind === "code") return importedCodePayload(asset, common);
  return importedFilePayload(asset, common);
}

function importedMarkdownPayload(asset: ImportedSpaceAsset): SpaceNodePayload {
  const { title, text } = extractImportedMarkdownDocument(asset.name, asset.text ?? "");
  return omitBy(
    {
      kind: "markdown",
      title,
      text,
      sourceRevision: asset.sourceRevision,
    },
    isUndefined,
  ) as SpaceNodePayload;
}

function importedAssetCommon(asset: ImportedSpaceAsset) {
  if (!asset.path) throw new Error("Imported resources require a Space-owned path");
  return omitBy(
    {
      title: asset.name,
      fileName: asset.name,
      assetPath: asset.path,
      sourceRevision: asset.sourceRevision,
      previewPath: asset.previewPath,
      previewText: asset.previewText,
      size: asset.size,
      modifiedAt: asset.modifiedAt,
      processingState: asset.processingState,
    },
    isUndefined,
  );
}

function importedVisualPayload(
  asset: ImportedSpaceAsset,
  common: ReturnType<typeof importedAssetCommon>,
): SpaceNodePayload {
  return omitBy(
    {
      ...common,
      kind: asset.kind as "image" | "video",
      intrinsicWidth: asset.intrinsicWidth,
      intrinsicHeight: asset.intrinsicHeight,
    },
    isUndefined,
  ) as SpaceNodePayload;
}

function importedCodePayload(
  asset: ImportedSpaceAsset,
  common: ReturnType<typeof importedAssetCommon>,
): SpaceNodePayload {
  if (asset.previewLanguage === undefined) throw new Error("Imported code requires a language");
  return omitBy(
    {
      ...common,
      kind: "code",
      previewLanguage: asset.previewLanguage,
      previewText: asset.previewText,
    },
    isUndefined,
  ) as SpaceNodePayload;
}

function importedFilePayload(
  asset: ImportedSpaceAsset,
  common: ReturnType<typeof importedAssetCommon>,
): SpaceNodePayload {
  return omitBy(
    {
      ...common,
      kind: "file",
      extractedText: asset.extractedText,
      intrinsicWidth: asset.intrinsicWidth,
      intrinsicHeight: asset.intrinsicHeight,
      entries: asset.entries,
    },
    isUndefined,
  ) as SpaceNodePayload;
}
