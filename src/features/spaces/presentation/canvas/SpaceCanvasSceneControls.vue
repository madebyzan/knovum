<script setup lang="ts">
import { ChevronLeft, ChevronRight, List } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed, nextTick, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import { removeSpaceAsset } from "../../assets/asset-operations";
import { saveCanvasThumbnailBlob } from "../../assets/canvas-thumbnail-file";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { SPACE_COMMAND_SHORTCUTS } from "../../commands/space-command-shortcuts";
import type { SpaceScene } from "../../scenes/scene";
import { useSceneNavigation } from "../../scenes/use-scene-navigation";
import { useSpaceStore } from "../../session/space-store";
import SpaceCanvasScenePanel from "./SpaceCanvasScenePanel.vue";

const { t } = useI18n();
const store = useSpaceStore();
const editorSession = useCanvasEditorSession();
const { scenes, activeSceneId, recentSceneId, createScene, getTargetScene, openScene, stepScene } =
  useSceneNavigation();
const open = shallowRef(false);
const saving = shallowRef(false);
const canStep = computed(() => scenes.value.length > 1);
const availableStructureNodeIds = computed(() => new Set(store.editSnapshot.structure.map(({ id }) => id)));

function sceneLabel(direction: -1 | 1) {
  const label = t(direction === -1 ? "spaces.scenes.previous" : "spaces.scenes.next");
  const target = canStep.value ? getTargetScene(direction) : undefined;
  return target ? `${label}: ${target.title}` : label;
}

async function saveScene() {
  if (saving.value) return;
  const editor = editorSession.activeEditor.value;
  if (!editor) throw new Error("Canvas editor is unavailable");
  saving.value = true;
  let thumbnail: string | undefined;
  try {
    thumbnail = await saveCanvasThumbnailBlob(await editor.captureSceneViewport());
    const created = await createScene(thumbnail);
    if (!created) await removeSpaceAsset(thumbnail);
    await nextTick();
  } catch (error) {
    if (thumbnail) await removeSpaceAsset(thumbnail);
    throw error;
  } finally {
    saving.value = false;
  }
}

async function selectScene(scene: SpaceScene) {
  await openScene(scene);
  open.value = false;
}

async function updateScene(scene: SpaceScene) {
  const viewport = editorSession.viewport.value;
  const editor = editorSession.activeEditor.value;
  if (!viewport || !editor) throw new Error("Canvas editor viewport is unavailable");
  const thumbnail = await saveCanvasThumbnailBlob(await editor.captureSceneViewport());
  let updated: SpaceScene | undefined;
  try {
    updated = await store.updateScene(scene.id, {
      viewport,
      thumbnailPath: thumbnail,
      captureView: true,
    });
  } catch (error) {
    await removeSpaceAsset(thumbnail);
    throw error;
  }
  if (updated) {
    store.announceScene(updated, "updated");
  } else {
    await removeSpaceAsset(thumbnail);
  }
}

async function duplicateScene(scene: SpaceScene) {
  const duplicate = await store.duplicateScene(scene.id);
  if (duplicate) {
    store.announceScene(duplicate, "saved");
  }
}
</script>

<template>
  <ToolbarRoot
    class="flex h-full cursor-default items-center gap-0.5 select-none"
    orientation="horizontal"
    :loop="false"
    :aria-label="t('spaces.scenes.title')"
  >
    <ToolbarButton as-child>
      <UiIconButton
        :label="sceneLabel(-1)"
        :shortcut="SPACE_COMMAND_SHORTCUTS.previousScene"
        size="sm"
        class="size-5 rounded-sm"
        :disabled="!canStep"
        @click="stepScene(-1)"
      >
        <ChevronLeft class="size-3" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <UiPopover v-model:open="open" side="top" align="center" :side-offset="6" class="p-0">
      <template #trigger>
        <ToolbarButton as-child>
          <UiIconButton :label="t('spaces.scenes.title')" tooltip size="sm" class="size-5 rounded-sm">
            <List class="size-3" :stroke-width="1.55" />
          </UiIconButton>
        </ToolbarButton>
      </template>
      <SpaceCanvasScenePanel
        :scenes="scenes"
        :available-structure-node-ids="availableStructureNodeIds"
        :active-scene-id="activeSceneId"
        :recent-scene-id="recentSceneId"
        :saving="saving"
        @create="saveScene"
        @duplicate="duplicateScene"
        @open="selectScene"
        @remove="store.removeScene($event.id)"
        @rename="(scene, title) => store.updateScene(scene.id, { title })"
        @update="updateScene"
      />
    </UiPopover>
    <ToolbarButton as-child>
      <UiIconButton
        :label="sceneLabel(1)"
        :shortcut="SPACE_COMMAND_SHORTCUTS.nextScene"
        size="sm"
        class="size-5 rounded-sm"
        :disabled="!canStep"
        @click="stepScene(1)"
      >
        <ChevronRight class="size-3" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
  </ToolbarRoot>
</template>
