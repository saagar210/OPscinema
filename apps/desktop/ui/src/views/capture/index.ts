import { ipc } from '../../ipc/typed';

export interface CaptureViewModel {
  status?: {
    state: string;
    session_id?: string;
  };
  keyframes: Array<{ frame_event_id: string; frame_ms: number; asset_id: string }>;
}

export async function loadCaptureView(sessionId: string | undefined): Promise<CaptureViewModel> {
  const status = await ipc.capture_get_status({ session_id: sessionId });

  if (!sessionId) {
    return {
      status: status.ok ? status.value : undefined,
      keyframes: [],
    };
  }

  const keyframes = await ipc.timeline_get_keyframes({
    session_id: sessionId,
    start_ms: 0,
    end_ms: Number.MAX_SAFE_INTEGER,
  });

  return {
    status: status.ok ? status.value : undefined,
    keyframes: keyframes.ok
      ? keyframes.value.keyframes.map((frame) => ({
          frame_event_id: frame.frame_event_id,
          frame_ms: frame.frame_ms,
          asset_id: frame.asset.asset_id,
        }))
      : [],
  };
}
