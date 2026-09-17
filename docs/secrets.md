# Secrets and credentials

Never put npm tokens, Git credentials, AI API keys or other secrets in Pinia, Tauri Store, localStorage or ordinary SQLite columns.

Knovum stores product secrets through the maintained Rust `keyring` crate so macOS uses Keychain Services and Windows uses Windows Credential Manager. The WebView only sees three product commands (`secrets_get_secret`, `secrets_set_secret`, `secrets_delete_secret`); it never receives a generic credential-store capability.

`foundation/secrets/secret-store.ts` is intentionally a domain boundary rather than a rename-only vendor wrapper: it is the only guest entry point to the product-owned Rust commands and preserves one key namespace/security policy.

Rules:

- keys are stable product identifiers, not user-entered paths;
- never log secret values, auth headers, credential URLs or environment dumps;
- remove credentials when an account/integration is disconnected;
- migration/rotation belongs to the owning integration;
- UI errors may identify the integration but must not echo secret material.

## Spaces sensitive-information nodes

Spaces supports two credential node types: **API Key** and **Account & password**. Their SQLite node payload contains only the title, credential type and a configured-status flag. The API key, username and password are serialized only into the OS credential store under a key derived from the node UUID.

The editor loads plaintext only while the credential dialog is open and clears those Vue refs when it closes. Clipboard/duplicate operations copy only the non-secret node metadata and reset the configured flag, so duplicating a card never duplicates its secret. Search/FTS indexes the title only. A normal node delete is a soft delete so undo can restore the node; its credential-store entry is therefore retained until the credential is explicitly cleared or the Space is permanently deleted. Permanent Space deletion removes the credential-store entries first and restores backups if either credential cleanup or database deletion fails, preventing a half-deleted Space from silently losing credentials.

Credential nodes must never gain `text`, `plainText`, `document`, URL or asset fields as an alternate persistence route. The item schema rejects those combinations.
