import { ipc } from '../../ipc/typed';
import type { AppResult, JsonObject } from '../../ipc/generated';

export interface StepsViewModel {
  steps: Array<{ step_id: string; title: string; order_index: number }>;
  head_seq: number;
}

export async function loadStepsView(sessionId: string | undefined): Promise<StepsViewModel> {
  if (!sessionId) {
    return { steps: [], head_seq: 0 };
  }

  const steps = await ipc.steps_list({
    session_id: sessionId,
  });

  if (!steps.ok) {
    return {
      steps: [],
      head_seq: 0,
    };
  }

  return steps.value;
}

export async function applyStepEditWithConflictRetry(
  sessionId: string,
  baseSeq: number,
  op: JsonObject,
): Promise<AppResult<{ head_seq: number; applied: boolean }>> {
  const initial = await ipc.steps_apply_edit({
    session_id: sessionId,
    base_seq: baseSeq,
    op,
  });

  if (initial.ok || initial.error.code !== 'CONFLICT') {
    return initial;
  }

  const refreshed = await ipc.steps_list({
    session_id: sessionId,
  });

  if (!refreshed.ok) {
    return refreshed;
  }

  return ipc.steps_apply_edit({
    session_id: sessionId,
    base_seq: refreshed.value.head_seq,
    op,
  });
}
