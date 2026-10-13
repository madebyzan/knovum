import { open } from "@tauri-apps/plugin-dialog";
import { computed, onScopeDispose, shallowRef, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { importSpaceBatch } from "../../assets/import-batch";
import { spaceImportContext } from "../../assets/import-context";
import { useSpaceStore } from "../../session/space-store";
import { insertMaterial } from "../edits/material";
import { isVideoMaterial, materialMatches, type VideoMaterialNode } from "../material";
import type { VideoEditProject } from "../project";
import { readVideoMaterial } from "../read-material";

const MEDIA_EXTENSIONS = [
  "mp4",
  "mov",
  "mkv",
  "webm",
  "png",
  "jpg",
  "jpeg",
  "webp",
  "mp3",
  "wav",
  "m4a",
  "flac",
  "ogg",
];

/** Space media available to the editor, and the two ways it enters the timeline. */
export function useVideoMaterials(
  spaceId: string,
  project: Ref<VideoEditProject | undefined>,
  commit: (project: VideoEditProject) => void,
) {
  const store = useSpaceStore();
  const { t } = useI18n();
  const busy = shallowRef(false);
  const controller = new AbortController();
  onScopeDispose(() => controller.abort());
  const nodes = computed(() => (store.activeSpaceId === spaceId ? store.nodes.filter(isVideoMaterial) : []));
  const byId = computed(() => new Map(nodes.value.map((node) => [node.id, node])));
  const available = computed(
    () =>
      new Map(
        project.value?.assets.flatMap((asset) => {
          const node = byId.value.get(asset.nodeId);
          return materialMatches(asset, node) ? [[asset.nodeId, node] as const] : [];
        }) ?? [],
      ),
  );

  async function add(node: VideoMaterialNode, atMs: number) {
    if (busy.value || !project.value || controller.signal.aborted) return;
    busy.value = true;
    try {
      const asset = await readVideoMaterial(node, controller.signal);
      const current = project.value;
      if (!current || controller.signal.aborted || !materialMatches(asset, byId.value.get(node.id))) return;
      const next = insertMaterial(current, asset, atMs);
      if (next === current) {
        toast.error(t("spaces.media.videoEditor.materials.noRoom"));
        return;
      }
      commit(next);
    } catch (error) {
      if (!controller.signal.aborted) toast.error(getErrorMessage(error));
    } finally {
      busy.value = false;
    }
  }

  async function importFiles() {
    if (busy.value) return;
    busy.value = true;
    try {
      const paths = await open({
        multiple: true,
        directory: false,
        filters: [{ name: t("spaces.media.videoEditor.materials.title"), extensions: MEDIA_EXTENSIONS }],
      });
      if (!paths || controller.signal.aborted) return;
      const sourcePaths = typeof paths === "string" ? [paths] : paths;
      const outcomes = await importSpaceBatch({
        context: spaceImportContext(),
        spaceId,
        requests: sourcePaths.map((path) => ({ key: path, path, sourceAccess: "scoped" })),
        origin: store.nextInsertionPoint(),
        labels: {
          title: t("spaces.media.videoEditor.materials.import"),
          preparing: t("spaces.media.videoEditor.materials.preparing"),
        },
      });
      for (const outcome of outcomes)
        if (outcome.status === "failed") toast.error(getErrorMessage(outcome.error));
    } catch (error) {
      if (!controller.signal.aborted) toast.error(getErrorMessage(error));
    } finally {
      busy.value = false;
    }
  }

  return { nodes, available, busy, add, importFiles };
}
