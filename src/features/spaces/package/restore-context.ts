import type { PackageIdMaps } from "./id-maps";
import type { SpacePackageManifest } from "./manifest-schema";

export interface SpacePackageRestoreContext {
  manifest: SpacePackageManifest;
  maps: PackageIdMaps;
  assets: Map<string, string>;
}
