import { shallowRef, type Ref } from "vue";
import { toast } from "vue-sonner";

import { admitAiRequirements } from "../../../foundation/ai/ai-readiness-admission";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceMediaAnalysisLabels } from "./actions";
import { spaceMediaAnalysisLabels } from "./analysis-labels";

export function useMediaInsights(
  node: Ref<SpaceCanvasNode>,
  canGenerate: () => boolean,
  generate: (nodeId: string, labels: SpaceMediaAnalysisLabels, target: TaskTarget) => Promise<void>,
) {
  const insightsTarget = shallowRef<TaskTarget>();

  async function generateInsights(target?: TaskTarget) {
    if (!canGenerate()) return;
    if (!target) {
      await admitAiRequirements([{ kind: "routed", purpose: "knowledge" }]);
      return;
    }
    try {
      await generate(node.value.id, spaceMediaAnalysisLabels(), target);
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  return { insightsTarget, generateInsights };
}
