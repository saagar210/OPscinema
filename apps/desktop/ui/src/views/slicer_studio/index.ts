import { ipc } from '../../ipc/typed';

export interface SlicerStudioViewModel {
  strict_ready: boolean;
  issues: string[];
}

export async function loadSlicerStudioView(sessionId: string | undefined): Promise<SlicerStudioViewModel> {
  if (!sessionId) {
    return {
      strict_ready: false,
      issues: ['No active session'],
    };
  }

  const validate = await ipc.tutorial_validate_export({ session_id: sessionId });

  if (!validate.ok) {
    return {
      strict_ready: false,
      issues: [validate.error.message],
    };
  }

  return {
    strict_ready: validate.value.allowed,
    issues: validate.value.reasons,
  };
}
