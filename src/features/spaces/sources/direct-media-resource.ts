const DIRECT_VIDEO_EXTENSION = /\.(?:mp4|m4v|mov|webm)$/iu;
const DIRECT_AUDIO_EXTENSION = /\.(?:mp3|m4a|wav|ogg|flac|aac)$/iu;

/** What a link to a media file names itself as, read from the file it points at. */
export function directMediaResourceKind(url: URL): "audio" | "video" | undefined {
  if (DIRECT_VIDEO_EXTENSION.test(url.pathname)) return "video";
  if (DIRECT_AUDIO_EXTENSION.test(url.pathname)) return "audio";
  return undefined;
}
