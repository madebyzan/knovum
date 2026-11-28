import { open } from "@tauri-apps/plugin-dialog";
import { sortBy } from "es-toolkit";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { systemSansFamily } from "../../../../foundation/typography/system-font";
import { safeExportFileName } from "../../assets/export/naming";
import { saveSpaceTextFile } from "../../assets/export/text";
import { formatSubtitleSrt, formatSubtitleVtt, parseSubtitleCues } from "../../media/subtitle-codec";
import { readSubtitleFile } from "../../media/subtitle-io";
import {
  defaultCaptionStyle,
  generateCaptionsFromTranscripts,
  importSubtitlesIntoProject,
} from "../edits/captions";
import type { VideoEditor } from "./use-video-editor";

export function useVideoCaptions(editor: VideoEditor) {
  const { t } = useI18n();
  const { project, materials, data, commit, selection } = editor;

  const hasTranscript = computed(() => {
    const current = project.value;
    if (!current) return false;
    return current.videoTrack.clips.some((clip) => {
      const material = materials.available.value.get(clip.assetId);
      const knowledge = material?.data.kind === "video" ? material.data.mediaKnowledge : undefined;
      return Boolean(
        knowledge &&
        knowledge.sourceRevision === material?.data.sourceRevision &&
        knowledge.transcript.segments.length > 0,
      );
    });
  });

  function generateFromTranscript() {
    const current = project.value;
    if (!current) return;
    const style = defaultCaptionStyle(systemSansFamily());
    const result = generateCaptionsFromTranscripts(current, materials.available.value, style);
    if (!result.count) {
      toast.info(t("spaces.media.videoEditor.captions.fromTranscriptEmpty"));
      return;
    }
    commit(result.project);
    const first = result.project.textTrack.items[0];
    if (first) selection.select({ kind: "text", id: first.id }, { reveal: false });
    toast.success(t("spaces.media.videoEditor.captions.fromTranscriptDone", { count: result.count }));
  }

  async function importSubtitles() {
    const current = project.value;
    if (!current) return;
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Subtitles", extensions: ["srt", "vtt"] }],
      });
      if (typeof selected !== "string") return;
      const raw = await readSubtitleFile(selected);
      const cues = parseSubtitleCues(raw);
      const style = defaultCaptionStyle(systemSansFamily());
      const result = importSubtitlesIntoProject(current, cues, style);
      if (!result.count) return;
      commit(result.project);
      const first = result.project.textTrack.items[0];
      if (first) selection.select({ kind: "text", id: first.id }, { reveal: false });
      toast.success(t("spaces.media.videoEditor.captions.importDone", { count: result.count }));
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  async function exportSubtitles(format: "srt" | "vtt") {
    const items = project.value?.textTrack.items ?? [];
    if (!items.length) return;
    const sorted = sortBy(items, [(item) => item.startMs, (item) => item.endMs]);
    const cues = sorted.map((item) => ({ startMs: item.startMs, endMs: item.endMs, text: item.text }));
    const text = format === "vtt" ? formatSubtitleVtt(cues) : formatSubtitleSrt(cues);
    const baseName = safeExportFileName(data.value.title) || "subtitles";
    try {
      await saveSpaceTextFile({
        defaultPath: `${baseName}.${format}`,
        filters: [{ name: format.toUpperCase(), extensions: [format] }],
        text,
      });
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  return { hasTranscript, generateFromTranscript, importSubtitles, exportSubtitles };
}
