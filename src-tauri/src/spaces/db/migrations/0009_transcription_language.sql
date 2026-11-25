-- Allow explicit media transcription requests to freeze a user-chosen ISO-639 language code
-- (NULL preserves automatic first-window language detection).
ALTER TABLE spaces_media_transcription_requests
ADD COLUMN language TEXT;
