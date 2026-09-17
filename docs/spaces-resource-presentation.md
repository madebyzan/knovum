# Spaces resource presentation

This document defines how Knovum turns web media URLs and local documents into canvas resources.
General Spaces behavior, desktop policy, and dependency policy remain authoritative in their existing
documents.

## Product contract

Pasting a URL is capture. Knovum submits each HTTP(S) candidate to the Rust media importer before
choosing the durable node:

- a URL supported by yt-dlp downloads into a managed local `video` or `audio` asset;
- an unsupported HTTP(S) URL becomes the existing saved-website node;
- text that is not a URL becomes a note.

Web origin is metadata on a standard media asset, not a separate content type. Playback always reads
the managed local copy. The canonical page URL, provider, and provider-owned identifier remain in the
media node's `origin` metadata for attribution and navigation.

yt-dlp is the single web media extractor. Knovum distributes exact-pinned upstream executables and
invokes them directly from Rust. It does not duplicate yt-dlp's site registry, read browser cookies or
ambient configuration, load plugins or remote components, self-update, or receive shell text from the
WebView.

Local documents use the operating system for visual representation:

- macOS thumbnails: `QLThumbnailGenerator`;
- macOS full preview: `QLPreviewPanel`;
- Windows thumbnails: `IShellItemImageFactory` with `THUMBNAILONLY`;
- Windows full preview: a feature-owned native `IPreviewHandler` host.

Missing native preview support is reported as unavailable. Knovum does not substitute a custom Office
renderer or a misleading document image.

## Interaction model

Imported media reuses the standard canvas media node. Its resting state is a low-cost poster; it does
not mount a player while inactive or off-screen. Playback starts only after explicit user activation.
Closing the Space or leaving the visible canvas releases the player.

The Rust import workflow owns download progress, cancellation, isolated staging, and publication into
the managed asset directory. The media file and optional poster participate in the normal asset copy,
package, and cleanup lifecycle.

Document nodes show the OS-generated page thumbnail from Knovum's private preview cache. Thumbnail
generation is asynchronous. Double-click opens the associated application; the explicit Preview action
opens the native system preview when the platform provides one.

## Architecture

```text
Canvas paste
  -> sources/capture-url.ts
      -> media/import/workflow.ts
          -> plugin:spaces|import_media_from_url + progress Channel
              -> spaces/assets/import/media/workflow.rs
                  -> exact-pinned yt-dlp runtime
                  -> isolated download workspace
                  -> standard managed video/audio asset
      -> website/capture.ts when yt-dlp reports unsupported

Managed document import
  -> assets import/enrichment
      -> previews/thumbnail
          -> macOS Quick Look adapter
          -> Windows Shell image adapter
      -> native preview
          -> macOS QLPreviewPanel adapter
          -> Windows IPreviewHandler host
```

Vue owns presentation and activation state. Rust owns process execution, managed paths, native
frameworks, and generated files. IPC passes one strict import result; the persisted media schema is the
shared durable contract.

### Standard media payload

URL imports produce `video` or `audio` payloads with:

- `assetPath`, `fileName`, `size`, `modifiedAt`, `sourceRevision`: managed local identity;
- `title`, optional `creator`, optional `durationSeconds`, and video dimensions: normalized presentation metadata;
- optional `previewPath`: managed poster;
- `origin: { kind: "web", url, provider, externalId }`: provider-neutral source identity.

There is no remote playback URL, playback session, extractor-specific node kind, or compatibility
reader for the removed development-only schema.

### yt-dlp process boundary

The adapter uses tools located through `local_resources/tools/mod.rs`. Execution disables ambient
configuration, plugins, remote components, self-update, and playlists, and passes the validated URL
after `--`. Rust owns bounded process output, progress, cancellation, staging, and atomic publication.

The exact yt-dlp and Deno versions, executable hashes, sources, and license material are recorded in
`media-import-manifest.json` and validated by release packaging. YouTube JavaScript execution uses the
bundled Deno executable selected through an application-owned absolute path.

## Performance and lifecycle

- yt-dlp and thumbnail generation never run on the WebView thread.
- Only visible active media nodes mount playback elements.
- URL import progress uses the existing Task Center and Tauri Channel contract.
- Downloaded media and posters are copied, packaged, and removed through the standard managed-asset
  lifecycle.
- Document thumbnails are keyed by managed-file identity, modification time, requested size, and
  generator version; writes are atomic.

## Acceptance criteria

- A supported URL creates one fully downloaded standard `video` or `audio` node.
- YouTube, Bilibili, TikTok, X/Twitter, Instagram, Facebook, Vimeo, Twitch, SoundCloud, and other
  upstream extractors use the same yt-dlp workflow.
- An unsupported URL retains saved-website behavior; an import failure is surfaced and is not
  reinterpreted as a website.
- Playback has no network dependency after import.
- No extractor-specific media node, compatibility alias, or separate renderer remains.
- Document nodes use real OS thumbnails and native full preview where available.
- Semantic extraction, search, and RAG do not depend on visual preview availability.

## Upstream evidence

- yt-dlp integration and licensing: https://github.com/yt-dlp/yt-dlp
- Tauri process model and sidecars: https://v2.tauri.app/concept/process-model/ and
  https://v2.tauri.app/develop/sidecar/
- Apple Quick Look Thumbnailing: https://developer.apple.com/documentation/quicklookthumbnailing/qlthumbnailgenerator
- Apple Quick Look panel: https://developer.apple.com/documentation/quicklookui/qlpreviewpanel
- Windows Shell thumbnails: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitemimagefactory-getimage
- Windows preview handlers: https://learn.microsoft.com/en-us/windows/win32/shell/preview-handlers
