CREATE TABLE plugins (id TEXT PRIMARY KEY, document TEXT NOT NULL CHECK(json_valid(document)));
CREATE TABLE skills (id TEXT PRIMARY KEY, document TEXT NOT NULL CHECK(json_valid(document)));
CREATE TABLE connections (id TEXT PRIMARY KEY, document TEXT NOT NULL CHECK(json_valid(document)));
CREATE TABLE connectors (id TEXT PRIMARY KEY, document TEXT NOT NULL CHECK(json_valid(document)));
CREATE TABLE connector_accounts (id TEXT PRIMARY KEY, document TEXT NOT NULL CHECK(json_valid(document)));
CREATE TABLE marketplaces (id TEXT PRIMARY KEY, document TEXT NOT NULL CHECK(json_valid(document)));
CREATE TABLE content_leases (snapshot_id TEXT PRIMARY KEY, document TEXT NOT NULL CHECK(json_valid(document)));
CREATE UNIQUE INDEX plugin_publication_identity ON plugins(json_extract(document,'$.publication.marketplaceId'),json_extract(document,'$.publication.publisherId'),json_extract(document,'$.publication.pluginName')) WHERE json_extract(document,'$.publication') IS NOT NULL;
