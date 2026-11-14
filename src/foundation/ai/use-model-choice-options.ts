import { storeToRefs } from "pinia";
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";

import type { LocalModelCapability } from "../../shared/contracts/generated/LocalModelCapability";
import type { LocalModelId } from "../../shared/contracts/generated/LocalModelId";
import type { RoutingPurpose } from "../../shared/contracts/generated/RoutingPurpose";
import type { TaskTarget } from "../../shared/contracts/generated/TaskTarget";
import { useLocalModelsQuery } from "../runtime/use-local-models-query";
import { availableExecutionTargets, defaultExecutionTarget } from "./ai-execution";
import { useAiSettingsStore } from "./ai-settings-store";
import { localModelChoiceOptions } from "./local-model-choice";
import { decideModelChoice, preferredChoiceId, type ModelChoiceOption } from "./model-choice";
import { useExecutionCatalogQuery } from "./use-execution-catalog-query";

/**
 * Local models ready to run for `capability`. `options` is non-empty only when 2+ models can run,
 * so single-click triggers can decide whether to run immediately or ask.
 */
export function useLocalModelOptions(
  capability: () => LocalModelCapability,
  enabled: () => boolean = () => true,
) {
  const catalog = useLocalModelsQuery(enabled);
  const decision = computed(() =>
    decideModelChoice(localModelChoiceOptions(catalog.data.value ?? [], capability())),
  );
  const options = computed<readonly ModelChoiceOption[]>(() =>
    decision.value.kind === "choose" ? decision.value.options : [],
  );
  const defaultId = computed<LocalModelId | undefined>(() => {
    const current = decision.value;
    if (current.kind === "run") return current.id as LocalModelId;
    if (current.kind === "choose") return current.selectedId as LocalModelId;
    return undefined;
  });
  return { options, defaultId };
}

/**
 * Routed AI targets available for `purpose`. `options` is non-empty only when 2+ targets exist,
 * keeping single-target actions one-click.
 */
export function useRoutedTargetOptions(purpose: () => RoutingPurpose) {
  const { t } = useI18n();
  const store = useAiSettingsStore();
  const catalog = useExecutionCatalogQuery({ detectInstallations: false });
  const { settings, ready } = storeToRefs(store);
  onMounted(() => void store.ensureLoaded());

  const allOptions = computed<ModelChoiceOption[]>(() => {
    if (!ready.value) return [];
    const currentPurpose = purpose();
    const configured = defaultExecutionTarget(settings.value, currentPurpose);
    const configuredKey = configured ? JSON.stringify(configured) : undefined;
    return availableExecutionTargets(settings.value, currentPurpose).map((option) => {
      const name =
        option.target.type === "agent"
          ? (catalog.data.value?.agents.find(
              ({ kind }) => option.target.type === "agent" && kind === option.target.runtimeKind,
            )?.displayName ?? option.label)
          : option.label;
      return {
        id: option.value,
        name,
        description: t(option.target.type === "model" ? "ai.target.modelApi" : "ai.target.desktopAgent"),
        runnable: true,
        preferred: option.value === configuredKey,
      };
    });
  });

  const options = computed<readonly ModelChoiceOption[]>(() =>
    allOptions.value.length > 1 ? allOptions.value : [],
  );
  const defaultTarget = computed<TaskTarget | undefined>(() => {
    const id = preferredChoiceId(allOptions.value);
    return id ? (JSON.parse(id) as TaskTarget) : undefined;
  });
  function targetForId(id: string): TaskTarget | undefined {
    return allOptions.value.some((option) => option.id === id) ? (JSON.parse(id) as TaskTarget) : undefined;
  }

  return { options, defaultTarget, targetForId };
}
