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

## Agent connector accounts

A service is connected once. An authorization is identified by the connector it belongs to, so
`connector_accounts` is keyed by connector, connections never select one, and a second connect
renews the same authorization instead of adding another. Account metadata lives in `extensions.db`;
tokens and SDK `StoredCredentials` live only in `secrets.db` under
`agent-extension:<connector id>`. The SDK CredentialStore adapter coordinates
refresh writes with account revocation. Manual credentials go directly through the authorization
command, with local UI fields cleared; query caches, settings, packages and graph checkpoints
contain no account secret values. OAuth uses public SDK discovery, PKCE, state validation and
an ephemeral loopback callback. Registered public client IDs are explicit; absent registration
support is a visible error. Accounts bind the exact resource endpoint and auth requirements.
Every successful authorization creates a new public account revision. Its commit also disables the
connector's connections and clears their tool permissions, because what the user approved was
approved against the authorization that has just been replaced. Task snapshots freeze that revision; reauthorization
requires fresh tool review and cannot substitute credentials in an existing checkpoint.

Disconnect first marks the account inactive with cleanup pending, cancels authorization and
sessions, then removes credentials under the same credential-write lock. Deletion failure leaves
the account inactive and visibly pending; startup retries deletion, including interrupted inactive
authorizations. Metadata cannot restore authorization during token refresh or OAuth completion.
MCP form elicitation cannot ask for credential fields; account authorization is separate.
The credentials database remains unencrypted and a copy exposes its values.

SDK CredentialStore instances capture the connector account revision when created. Load, save and clear share the per-account credential lock and require that same revision and current authority; only the captured, uncancelled authorization flow may access credentials before the public grant is enabled. An old client cannot read, overwrite or clear a renewed account’s credentials. Revocation cleanup remains an explicit owner operation.

An authorization attempt distinguishes two cancellations. The token registered for an account
means only that the user asked to stop, and the flow never cancels it, so an authorization that
finishes is never reported as cancelled. Shutting down the loopback callback and closing the window
in which credentials may be written uses a child token the attempt cancels itself. Starting an
attempt supersedes whatever attempt that account had in flight, and an attempt only deregisters
itself, so a cancel can only stop the attempt it was asked for. A provider that redirects back with
an `error` parameter is reported as a denial, because there is no authorization code to exchange.
