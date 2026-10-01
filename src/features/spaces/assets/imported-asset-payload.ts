import type { PreparedAsset as ImportedSpaceAsset } from "../contracts/generated/PreparedAsset";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { extractImportedMarkdownDocument } from "./markdown-document";

export function importedSpaceNodePayload(asset: ImportedSpaceAsset): SpaceNodePayload {
  if (asset.kind === "markdown") return importedMarkdownPayload(asset);
  const common = importedAssetCommon(asset);
  if (asset.kind === "image") return { ...common, kind: "image", ...intrinsicSize(asset) };
  if (asset.kind === "video") return { ...common, kind: "video", ...intrinsicSize(asset) };
  if (asset.kind === "audio") return { ...common, kind: "audio" };
  if (asset.kind === "code") return importedCodePayload(asset, common);
  return importedFilePayload(asset, common);
}

function importedMarkdownPayload(asset: ImportedSpaceAsset): SpaceNodePayload {
  const { title, text } = extractImportedMarkdownDocument(asset.name, asset.text ?? "");
  return {
    kind: "markdown",
    title,
    text,
    ...(asset.sourceRevision !== undefined && { sourceRevision: asset.sourceRevision }),
  };
}

function importedAssetCommon(asset: ImportedSpaceAsset) {
  if (!asset.path) throw new Error("Imported resources require a Space-owned path");
  return {
    title: asset.name,
    fileName: asset.name,
    assetPath: asset.path,
    ...(asset.sourceRevision !== undefined && { sourceRevision: asset.sourceRevision }),
    ...(asset.previewPath !== undefined && { previewPath: asset.previewPath }),
    ...(asset.previewText !== undefined && { previewText: asset.previewText }),
    ...(asset.size !== undefined && { size: asset.size }),
    ...(asset.modifiedAt !== undefined && { modifiedAt: asset.modifiedAt }),
    ...(asset.processingState !== undefined && { processingState: asset.processingState }),
  };
}

function intrinsicSize(asset: ImportedSpaceAsset) {
  return {
    ...(asset.intrinsicWidth !== undefined && { intrinsicWidth: asset.intrinsicWidth }),
    ...(asset.intrinsicHeight !== undefined && { intrinsicHeight: asset.intrinsicHeight }),
  };
}

function importedCodePayload(
  asset: ImportedSpaceAsset,
  common: ReturnType<typeof importedAssetCommon>,
): SpaceNodePayload {
  if (asset.previewLanguage === undefined) throw new Error("Imported code requires a language");
  return {
    ...common,
    kind: "code",
    previewLanguage: asset.previewLanguage,
  };
}

function importedFilePayload(
  asset: ImportedSpaceAsset,
  common: ReturnType<typeof importedAssetCommon>,
): SpaceNodePayload {
  return {
    ...common,
    kind: "file",
    ...(asset.extractedText !== undefined && { extractedText: asset.extractedText }),
    ...intrinsicSize(asset),
    ...(asset.entries !== undefined && { entries: asset.entries }),
  };
}
