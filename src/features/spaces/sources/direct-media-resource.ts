const DIRECT_MEDIA_EXTENSION = /\.(?:mp4|m4v|mov|webm|mp3|m4a|wav|ogg|flac|aac)$/iu;

export function isDirectMediaResource(url: URL): boolean {
  return DIRECT_MEDIA_EXTENSION.test(url.pathname);
}
