import { useToggle } from "@vueuse/core";
import { computed, onScopeDispose, shallowRef, watch } from "vue";
import { toast } from "vue-sonner";

import { errorMessage } from "../../../foundation/errors/error-format";
import type { SpaceCredentialPayload } from "../model/nodes/node-payload";
import { commitSpaceCredentialChange } from "./credential-actions";
import { loadSpaceCredential } from "./credential-storage";

interface CredentialEditorOptions {
  id: string;
  data: SpaceCredentialPayload;
  persist: (id: string, data: SpaceCredentialPayload) => Promise<void>;
}

export function useCredentialEditor(options: CredentialEditorOptions) {
  const open = shallowRef(false);
  const loading = shallowRef(false);
  const loaded = shallowRef(false);
  const saving = shallowRef(false);
  let request = 0;
  const reveal = shallowRef(false);
  const toggleReveal = useToggle(reveal);
  const title = shallowRef("");
  const secret = shallowRef("");
  const username = shallowRef("");
  const password = shallowRef("");
  const hasCredentialDraft = computed(() => Boolean(credentialDraft()));
  const hasEmptyDraft = computed(() =>
    options.data.credentialKind === "api-key"
      ? !secret.value.trim()
      : !username.value.trim() && !password.value,
  );
  const canSave = computed(
    () =>
      !loading.value &&
      !saving.value &&
      Boolean(title.value.trim()) &&
      (hasCredentialDraft.value || (loaded.value && hasEmptyDraft.value)),
  );

  watch(
    open,
    (value) => {
      if (!value) {
        request += 1;
        loading.value = false;
        clearDraft();
      }
    },
    { flush: "sync" },
  );
  onScopeDispose(() => {
    request += 1;
    clearDraft();
  });

  async function openEditor() {
    if (open.value || saving.value) return;
    const currentRequest = ++request;
    clearDraft();
    title.value = options.data.title;
    open.value = true;
    loading.value = true;
    try {
      const stored = await loadSpaceCredential(options.id);
      if (currentRequest !== request) return;
      applyStoredCredential(stored);
      loaded.value = Boolean(stored);
    } catch (error) {
      if (currentRequest !== request) return;
      open.value = false;
      toast.error(errorMessage(error));
    } finally {
      if (currentRequest === request) loading.value = false;
    }
  }

  async function save() {
    if (!canSave.value) return;
    const name = title.value.trim();
    const credential = credentialDraft();
    saving.value = true;
    try {
      await commitSpaceCredentialChange(options.id, credential, () =>
        options.persist(options.id, {
          ...options.data,
          title: name,
          credentialConfigured: Boolean(credential),
        }),
      );
      open.value = false;
    } catch (error) {
      toast.error(errorMessage(error));
    } finally {
      saving.value = false;
    }
  }

  function credentialDraft() {
    if (options.data.credentialKind === "api-key") {
      return secret.value.trim() ? { kind: "api-key" as const, secret: secret.value } : undefined;
    }
    return username.value.trim() && password.value
      ? { kind: "account-password" as const, username: username.value, password: password.value }
      : undefined;
  }

  function applyStoredCredential(value: Awaited<ReturnType<typeof loadSpaceCredential>>) {
    if (value?.kind === "api-key") secret.value = value.secret;
    if (value?.kind === "account-password") {
      username.value = value.username;
      password.value = value.password;
    }
  }

  function clearDraft() {
    reveal.value = false;
    loaded.value = false;
    secret.value = "";
    username.value = "";
    password.value = "";
    title.value = "";
  }

  return {
    open,
    loading,
    saving,
    reveal,
    title,
    secret,
    username,
    password,
    canSave,
    openEditor,
    toggleReveal,
    save,
  };
}
