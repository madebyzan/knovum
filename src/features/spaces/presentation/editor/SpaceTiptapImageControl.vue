<script setup lang="ts">
import { Check, ImagePlus, Link2, Upload } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import { chooseSpaceTiptapImages } from "../../editor/tiptap-images";
import { normalizeUrl } from "../../editor/url-validation";

const IMAGE_PROTOCOLS = ["http:", "https:", "data:", "asset:"] as const;

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();
const open = ref(false);
const tab = ref<"upload" | "url">("upload");
const url = ref("");
const alt = ref("");
const busy = ref(false);
const touched = ref(false);
const normalizedUrl = computed(() => normalizeUrl(url.value, IMAGE_PROTOCOLS));
const invalidUrl = computed(() => touched.value && Boolean(url.value.trim()) && !normalizedUrl.value);

async function choose() {
  if (busy.value) return;
  busy.value = true;
  try {
    if (await chooseSpaceTiptapImages(props.editor)) open.value = false;
  } catch (error) {
    toast.error(t("editor.imageUploadFailed"), { description: String(error) });
  } finally {
    busy.value = false;
  }
}

function insertUrl() {
  touched.value = true;
  if (!normalizedUrl.value) return;
  const inserted = props.editor
    .chain()
    .focus()
    .setImage({
      src: normalizedUrl.value,
      ...(alt.value.trim() ? { alt: alt.value.trim() } : {}),
    })
    .run();
  if (!inserted) {
    toast.error(t("editor.imageInsertFailed"));
    return;
  }
  url.value = "";
  alt.value = "";
  touched.value = false;
  open.value = false;
}
</script>

<template>
  <UiPopover
    v-model:open="open"
    align="start"
    :side-offset="7"
    class="w-72 overflow-hidden rounded-dialog p-0"
  >
    <template #trigger>
      <UiIconButton :label="t('editor.insertImage')" size="sm">
        <ImagePlus class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
      </UiIconButton>
    </template>

    <div class="flex border-b border-divider">
      <button
        type="button"
        class="flex flex-1 items-center justify-center gap-1.5 px-3 py-2 text-caption font-medium transition-colors outline-none"
        :class="
          tab === 'upload'
            ? 'border-b-2 border-accent text-foreground'
            : 'text-foreground-secondary hover:text-foreground'
        "
        @click="tab = 'upload'"
      >
        <Upload class="size-(--knovum-icon-size-xs)" :stroke-width="1.6" />{{ t("editor.imageTabUpload") }}
      </button>
      <button
        type="button"
        class="flex flex-1 items-center justify-center gap-1.5 px-3 py-2 text-caption font-medium transition-colors outline-none"
        :class="
          tab === 'url'
            ? 'border-b-2 border-accent text-foreground'
            : 'text-foreground-secondary hover:text-foreground'
        "
        @click="tab = 'url'"
      >
        <Link2 class="size-(--knovum-icon-size-xs)" :stroke-width="1.6" />{{ t("editor.imageTabUrl") }}
      </button>
    </div>

    <div v-if="tab === 'upload'" class="flex flex-col items-center gap-3 px-4 py-5">
      <button
        type="button"
        class="flex h-20 w-full flex-col items-center justify-center gap-1.5 rounded-control border-2 border-dashed border-divider text-foreground-secondary transition-colors hover:border-accent hover:text-foreground"
        :disabled="busy"
        @click="choose"
      >
        <Upload class="size-(--knovum-icon-size-md)" :stroke-width="1.4" />
        <span class="text-caption">{{ t("editor.chooseImage") }}</span>
      </button>
      <p class="text-micro text-foreground-tertiary">{{ t("editor.imageDropDocumentHint") }}</p>
    </div>

    <div v-else class="flex flex-col gap-2.5 px-3 py-3">
      <label class="grid gap-1.5">
        <span class="flex items-center gap-1.5 text-caption font-medium text-foreground-secondary">
          <Link2 class="size-(--knovum-icon-size-xs)" />{{ t("editor.imageUrl") }}
        </span>
        <UiInput
          v-model="url"
          type="url"
          placeholder="https://…"
          @input="touched = true"
          @keydown.enter.prevent="insertUrl"
        />
      </label>
      <p v-if="invalidUrl" class="-mt-1 text-caption text-danger">{{ t("editor.imageUrlInvalid") }}</p>
      <label class="grid gap-1.5">
        <span class="text-caption font-medium text-foreground-secondary">{{ t("editor.imageAlt") }}</span>
        <UiInput
          v-model="alt"
          :placeholder="t('editor.imageAltPlaceholder')"
          @keydown.enter.prevent="insertUrl"
        />
      </label>
      <div class="flex justify-end pt-0.5">
        <UiButton size="sm" variant="primary" :disabled="!normalizedUrl" @click="insertUrl">
          <Check class="size-(--knovum-icon-size-xs)" />{{ t("editor.insertImage") }}
        </UiButton>
      </div>
    </div>
  </UiPopover>
</template>
