import { computed, shallowRef } from "vue";
import {
  useRoute,
  useRouter,
  onBeforeRouteUpdate,
  isNavigationFailure,
  type LocationQueryRaw,
} from "vue-router";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useSettingsLeaveContext } from "../../../foundation/lifecycle/use-settings-leave-context";
import { isApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { isExtensionTab, type ExtensionDestination } from "./extension-destination";

export function useExtensionNavigation() {
  const route = useRoute();
  const router = useRouter();
  const activeTab = computed(() =>
    isExtensionTab(route.query.extensionTab) ? route.query.extensionTab : "plugins",
  );
  const extensionId = computed(() =>
    typeof route.query.extensionId === "string" ? route.query.extensionId : undefined,
  );
  const connectorId = computed(() =>
    typeof route.query.connectorId === "string" ? route.query.connectorId : undefined,
  );
  const action = computed(() =>
    route.query.extensionAction === "create" || route.query.extensionAction === "browse"
      ? route.query.extensionAction
      : undefined,
  );
  const isBrowsing = computed(() => !(extensionId.value || connectorId.value || action.value === "create"));
  const search = shallowRef("");
  const error = shallowRef("");
  const leaveCoordinator = useSettingsLeaveContext();

  function isCurrentDestination(destination: ExtensionDestination) {
    return (
      destination.tab === activeTab.value &&
      destination.id === extensionId.value &&
      destination.connectorId === connectorId.value &&
      destination.action === action.value
    );
  }
  async function navigate(destination: ExtensionDestination) {
    if (isCurrentDestination(destination)) return;
    try {
      const query: LocationQueryRaw = { ...route.query, extensionTab: destination.tab };
      delete query.extensionId;
      delete query.setting;
      delete query.connectorId;
      delete query.extensionAction;
      if (destination.connectorId) query.connectorId = destination.connectorId;
      if (destination.action) query.extensionAction = destination.action;
      if (destination.id) query.extensionId = destination.id;
      const navigation = await router.push({ path: route.path, query });
      if (isNavigationFailure(navigation)) return;
      search.value = "";
      error.value = "";
    } catch (cause) {
      if (!isApplicationLeaveCancelled(cause)) error.value = getErrorMessage(cause);
    }
  }
  onBeforeRouteUpdate(async (to, from) => {
    if (to.query.pane !== from.query.pane) return true;
    if (
      to.query.extensionTab === from.query.extensionTab &&
      to.query.extensionId === from.query.extensionId &&
      to.query.connectorId === from.query.connectorId &&
      to.query.extensionAction === from.query.extensionAction
    )
      return true;
    try {
      await leaveCoordinator.requestLeave();
      return true;
    } catch (cause) {
      if (!isApplicationLeaveCancelled(cause)) error.value = getErrorMessage(cause);
      return false;
    }
  });
  function selectTab(value: string | number) {
    if (isExtensionTab(value)) void navigate({ tab: value });
  }

  return { activeTab, extensionId, connectorId, action, isBrowsing, search, error, navigate, selectTab };
}
