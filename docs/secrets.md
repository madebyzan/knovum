# Secrets and credentials

Never put npm tokens, Git credentials, AI API keys or other secrets in Pinia, Tauri Store, localStorage or feature data tables. Store product credentials only in the dedicated `secrets.db` database through the secret commands.

Knovum stores product credentials as ordinary values in the foundation-owned `secrets.db` SQLite database under the application's configuration directory. The database is protected by the user's local account and disk security, not by application encryption. On Unix the database file is restricted to its owner. A copy of the database exposes its credentials; do not describe this as encrypted storage. The WebView reaches credentials only through the scoped `plugin:secrets` get, set and delete commands; it never receives raw SQL access to this database.

`foundation/secrets/secret-store.ts` is the only guest entry point to the product-owned Rust commands and preserves one key namespace and validation policy.

Rules:

- keys are stable product identifiers, not user-entered paths;
- never log secret values, auth headers, credential URLs or environment dumps;
- remove credentials when an account/integration is disconnected;
- migration/rotation belongs to the owning integration;
- UI errors may identify the integration but must not echo secret material.

## Spaces sensitive-information nodes

Spaces supports two credential node types: **API Key** and **Account & password**. Their Spaces database node payload contains only the title, credential type and a configured-status flag. The API key, username and password are serialized into `secrets.db` under a key derived from the node UUID.

The editor reads `secrets.db` whenever the credential dialog opens; the node's configured flag is metadata, not the authority for whether a value exists. It clears plaintext Vue refs when the dialog closes. Clipboard/duplicate operations copy only the non-secret node metadata and reset the configured flag, so duplicating a card never duplicates its secret. Search/FTS indexes the title only. A normal node delete is a soft delete so undo can restore the node; its credential database entry is retained until the credential is explicitly cleared or the Space is permanently deleted. Permanent Space deletion removes the credential entries first and restores backups if either credential cleanup or Space database deletion fails, preventing a half-deleted Space from silently losing credentials.

Credential nodes must never gain `text`, `plainText`, `document`, URL or asset fields as an alternate persistence route. The item schema rejects those combinations.

Capability checks that need a credential run when the user opens the relevant action, not when a canvas node or toolbar mounts. Credential database reads require no OS authorization dialog. The former Keychain-backed `secrets.vault` is not read or migrated by the new contract; users must re-enter credentials. Do not silently import obsolete credentials or delete the old file as a side effect of opening the new database.
