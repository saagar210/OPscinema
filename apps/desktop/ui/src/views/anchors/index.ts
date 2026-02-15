import { ipc } from '../../ipc/typed';

export interface AnchorsViewModel {
  step_id?: string;
  anchors: Array<{ anchor_id: string; degraded: boolean; confidence: number }>;
}

export async function loadAnchorsView(sessionId: string | undefined): Promise<AnchorsViewModel> {
  if (!sessionId) {
    return { anchors: [] };
  }

  const steps = await ipc.steps_list({ session_id: sessionId });

  if (!steps.ok || steps.value.steps.length === 0) {
    return { anchors: [] };
  }

  const stepId = steps.value.steps[0].step_id;
  const anchors = await ipc.anchors_list_for_step({
    session_id: sessionId,
    step_id: stepId,
  });

  return {
    step_id: stepId,
    anchors: anchors.ok ? anchors.value.anchors : [],
  };
}
