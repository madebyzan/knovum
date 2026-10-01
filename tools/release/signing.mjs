import { httpsUrl, requireValue } from "./config.mjs";

/** Platform trust is distinct from the Tauri update signature. Never ship ad-hoc releases. */
export function signingConfiguration(target, environment) {
  if (target.startsWith("macos-")) {
    const identity = requireValue(environment.APPLE_SIGNING_IDENTITY, "APPLE_SIGNING_IDENTITY");
    if (!identity.startsWith("Developer ID Application:")) {
      throw new Error("Direct macOS releases require a Developer ID Application identity");
    }
    for (const name of ["APPLE_API_ISSUER", "APPLE_API_KEY", "APPLE_API_KEY_PATH"]) {
      requireValue(environment[name], name);
    }
    return { macOS: { signingIdentity: identity } };
  }
  if (target.startsWith("windows-")) {
    const certificateThumbprint = requireValue(
      environment.KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT, "KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT",
    );
    if (!/^[0-9a-fA-F]{40}$/.test(certificateThumbprint)) {
      throw new Error("Windows signing requires a 40-character certificate thumbprint");
    }
    return { windows: {
      certificateThumbprint,
      digestAlgorithm: "sha256",
      timestampUrl: httpsUrl(environment.KNOVUM_WINDOWS_TIMESTAMP_URL, "KNOVUM_WINDOWS_TIMESTAMP_URL"),
      tsp: true,
    } };
  }
  throw new Error(`Unsupported production signing host: ${target}`);
}
