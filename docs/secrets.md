# Secrets and credentials

Never put npm tokens, Git credentials, AI API keys or other secrets in Pinia, Tauri Store, localStorage or ordinary SQLite columns.

Knovum stores product secrets in one encrypted vault. The maintained Rust `keyring` crate holds only its unlock key, using macOS Keychain Services or Windows Credential Manager. Rust loads that key at most once per process and keeps the unlocked vault in memory for the session. The WebView reaches secrets only through the scoped `plugin:secrets` get, set and delete commands; it never receives a generic credential-store capability.

`foundation/secrets/secret-store.ts` is intentionally a domain boundary rather than a rename-only vendor wrapper: it is the only guest entry point to the product-owned Rust commands and preserves one key namespace/security policy.

Rules:

- keys are stable product identifiers, not user-entered paths;
- never log secret values, auth headers, credential URLs or environment dumps;
- remove credentials when an account/integration is disconnected;
- migration/rotation belongs to the owning integration;
- UI errors may identify the integration but must not echo secret material.

## Spaces sensitive-information nodes

Spaces supports two credential node types: **API Key** and **Account & password**. Their SQLite node payload contains only the title, credential type and a configured-status flag. The API key, username and password are serialized only into the encrypted vault under a key derived from the node UUID.

The editor loads plaintext only while the credential dialog is open and clears those Vue refs when it closes. Clipboard/duplicate operations copy only the non-secret node metadata and reset the configured flag, so duplicating a card never duplicates its secret. Search/FTS indexes the title only. A normal node delete is a soft delete so undo can restore the node; its vault entry is therefore retained until the credential is explicitly cleared or the Space is permanently deleted. Permanent Space deletion removes the vault entries first and restores backups if either credential cleanup or database deletion fails, preventing a half-deleted Space from silently losing credentials.

Credential nodes must never gain `text`, `plainText`, `document`, URL or asset fields as an alternate persistence route. The item schema rejects those combinations.

Capability checks that need a credential run when the user opens the relevant action, not when a canvas node or toolbar mounts. They do not retry or refresh on window focus: an OS authorization dialog itself changes focus and must not trigger another vault read. macOS development and release executables require a stable code-signing identity. The development launcher signs every rebuilt executable before launch; an unsigned or ad-hoc identity would change Keychain authorization across builds. Do not weaken Keychain access controls or duplicate the unlock key outside the OS credential store.
