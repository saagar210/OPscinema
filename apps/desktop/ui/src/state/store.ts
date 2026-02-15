import type { AppErrorCode } from '../ipc/generated';

export interface JobStatusEnvelope {
  stream_seq: number;
  sent_at: string;
  payload: {
    job_id: string;
    status: 'QUEUED' | 'RUNNING' | 'SUCCEEDED' | 'FAILED' | 'CANCELLED';
  };
}

export interface CaptureStatusEnvelope {
  stream_seq: number;
  sent_at: string;
  payload: {
    state: 'IDLE' | 'CAPTURING' | 'STOPPED';
    session_id?: string;
  };
}

export interface UiState {
  active_route: string;
  active_session_id?: string;
  session_head_seq: Record<string, number>;
  jobs: Record<string, JobStatusEnvelope['payload']['status']>;
  capture_state: CaptureStatusEnvelope['payload']['state'];
  invalidated: {
    jobs: boolean;
    capture: boolean;
    session: boolean;
  };
  last_error_code?: AppErrorCode;
}

export interface UiStore {
  getState(): UiState;
  setActiveRoute(route: string): void;
  setActiveSession(sessionId: string | undefined): void;
  setSessionHeadSeq(sessionId: string, nextHeadSeq: number): void;
  ingestJobStatus(event: JobStatusEnvelope): void;
  ingestCaptureStatus(event: CaptureStatusEnvelope): void;
  clearInvalidations(): void;
  setLastError(code: AppErrorCode | undefined): void;
}

export function createUiStore(): UiStore {
  const state: UiState = {
    active_route: 'permissions',
    session_head_seq: {},
    jobs: {},
    capture_state: 'IDLE',
    invalidated: {
      jobs: false,
      capture: false,
      session: false,
    },
  };

  return {
    getState(): UiState {
      return state;
    },
    setActiveRoute(route: string): void {
      if (state.active_route !== route) {
        state.active_route = route;
      }
    },
    setActiveSession(sessionId: string | undefined): void {
      if (state.active_session_id !== sessionId) {
        state.active_session_id = sessionId;
        state.invalidated.session = true;
      }
    },
    setSessionHeadSeq(sessionId: string, nextHeadSeq: number): void {
      const prev = state.session_head_seq[sessionId];
      if (prev !== nextHeadSeq) {
        state.session_head_seq[sessionId] = nextHeadSeq;
        state.invalidated.session = true;
      }
    },
    ingestJobStatus(event: JobStatusEnvelope): void {
      state.jobs[event.payload.job_id] = event.payload.status;
      state.invalidated.jobs = true;
    },
    ingestCaptureStatus(event: CaptureStatusEnvelope): void {
      state.capture_state = event.payload.state;
      if (event.payload.session_id) {
        state.active_session_id = event.payload.session_id;
      }
      state.invalidated.capture = true;
    },
    clearInvalidations(): void {
      state.invalidated.jobs = false;
      state.invalidated.capture = false;
      state.invalidated.session = false;
    },
    setLastError(code: AppErrorCode | undefined): void {
      state.last_error_code = code;
    },
  };
}
