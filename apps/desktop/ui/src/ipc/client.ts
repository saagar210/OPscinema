import type { AppError, AppErrorCode, AppResult, IpcCommand } from './generated';

export interface TypedIpcClient {
  invoke<TReq, TRes>(command: IpcCommand, payload: TReq): Promise<AppResult<TRes>>;
}

export type RuntimeEventUnlisten = () => void;

type InvokeFn = <TReq, TRes>(command: IpcCommand, payload: TReq) => Promise<AppResult<TRes>>;

let runtimeInvoke: InvokeFn | null = null;
const NO_REQ_COMMANDS = new Set<IpcCommand>([
  'app_get_build_info',
  'app_get_permissions_status',
  'settings_get',
  'network_allowlist_get',
  'capture_get_config',
  'model_roles_get',
  'agent_pipelines_list',
]);

export function setIpcRuntimeInvoke(invokeFn: InvokeFn): void {
  runtimeInvoke = invokeFn;
}

async function resolveTauriInvoke(): Promise<((command: string, payload?: unknown) => Promise<unknown>) | null> {
  const win = window as Window & {
    __TAURI__?: {
      core?: {
        invoke?: (command: string, payload?: unknown) => Promise<unknown>;
      };
    };
    __TAURI_INTERNALS__?: {
      invoke?: (command: string, payload?: unknown) => Promise<unknown>;
    };
  };
  return win.__TAURI__?.core?.invoke ?? win.__TAURI_INTERNALS__?.invoke ?? null;
}

export async function subscribeRuntimeEvent(
  eventName: 'job_status' | 'job_progress' | 'capture_status',
  onEvent: (payload: unknown) => void,
): Promise<RuntimeEventUnlisten> {
  const win = window as Window & {
    __TAURI__?: {
      event?: {
        listen?: (
          event: string,
          cb: (payload: { payload: unknown }) => void,
        ) => Promise<() => void>;
      };
    };
  };
  const listen = win.__TAURI__?.event?.listen;
  if (!listen) {
    return () => {};
  }
  const unlisten = await listen(eventName, (event) => {
    onEvent(event.payload);
  });
  return unlisten;
}

export const client: TypedIpcClient = {
  async invoke<TReq, TRes>(command: IpcCommand, payload: TReq): Promise<AppResult<TRes>> {
    if (runtimeInvoke) {
      return runtimeInvoke<TReq, TRes>(command, payload);
    }

    const tauriInvoke = await resolveTauriInvoke();
    if (!tauriInvoke) {
      return {
        ok: false,
        error: {
          code: 'INTERNAL',
          message: 'Tauri IPC runtime is unavailable',
          recoverable: false,
        },
      };
    }

    try {
      const invokePayload = NO_REQ_COMMANDS.has(command)
        ? (payload as unknown)
        : ({ req: payload } as unknown);
      const raw = await tauriInvoke(command, invokePayload);
      if (isAppResult(raw)) {
        return raw as AppResult<TRes>;
      }
      return {
        ok: true,
        value: raw as TRes,
      };
    } catch (err: unknown) {
      return {
        ok: false,
        error: toAppError(err),
      };
    }
  },
};

function isAppResult(value: unknown): value is AppResult<unknown> {
  if (!value || typeof value !== 'object') {
    return false;
  }
  const v = value as Record<string, unknown>;
  return typeof v.ok === 'boolean';
}

function toAppError(err: unknown): AppError {
  if (err && typeof err === 'object') {
    const e = err as Record<string, unknown>;
    if (typeof e.code === 'string' && typeof e.message === 'string') {
      return {
        code: e.code as AppErrorCode,
        message: e.message,
        details: typeof e.details === 'string' ? e.details : undefined,
        recoverable: typeof e.recoverable === 'boolean' ? e.recoverable : false,
        action_hint: typeof e.action_hint === 'string' ? e.action_hint : undefined,
      };
    }
  }
  return {
    code: 'INTERNAL',
    message: String(err ?? 'Unknown IPC error'),
    recoverable: false,
  };
}
