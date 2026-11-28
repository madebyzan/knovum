import { computed, type ComputedRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { safeExportFileName } from "../../assets/export/naming";
import { saveSpaceTextFile } from "../../assets/export/text";
import type { SpaceTranscriptSegment } from "../../media/schema/transcript";
import type { SpeakerBadge } from "../../media/speaker-display";
import { formatTranscriptPlainText, formatTranscriptSrt } from "../../media/transcript-export";

export function useTranscriptExport(
  title: () => string,
  segments: ComputedRef<readonly SpaceTranscriptSegment[]>,
  speakers: ComputedRef<ReadonlyMap<string, SpeakerBadge>>,
) {
  const { t } = useI18n();

  async function copyTranscript() {
    try {
      await navigator.clipboard.writeText(formatTranscriptPlainText(segments.value, speakers.value));
      toast.success(t("spaces.transcript.text.copied"));
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  async function exportTranscript(format: "txt" | "srt") {
    const baseName = safeExportFileName(title()) || "transcript";
    const text =
      format === "srt"
        ? formatTranscriptSrt(segments.value, speakers.value)
        : formatTranscriptPlainText(segments.value, speakers.value);
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

  const exportItems = computed(() => [
    { id: "txt", label: t("spaces.transcript.text.exportTxt"), onSelect: () => void exportTranscript("txt") },
    { id: "srt", label: t("spaces.transcript.text.exportSrt"), onSelect: () => void exportTranscript("srt") },
  ]);

  return { copyTranscript, exportItems };
}
