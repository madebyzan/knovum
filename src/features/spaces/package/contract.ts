export const SPACE_PACKAGE_FORMAT = "knovum-space" as const;
export const SPACE_PACKAGE_VERSION = 1 as const;
export const PACKAGE_ASSET_PREFIX = "knovum-package-asset:";

export interface SpacePackageAsset {
  token: string;
  path: string;
}
export interface ImportedSpacePackageAsset {
  token: string;
  path: string;
}
export interface ImportedSpacePackage {
  manifest: string;
  assets: ImportedSpacePackageAsset[];
}
