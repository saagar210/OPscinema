import { ipc } from '../ipc/typed';
import type { AppResult } from '../ipc/generated';
import { createUiStore } from '../state/store';
import { loadAgentPlantView } from '../views/agent_plant';
import { loadAnchorsView } from '../views/anchors';
import { loadCaptureView } from '../views/capture';
import { loadEvidenceView } from '../views/evidence';
import { loadModelDockView } from '../views/model_dock';
import { loadPermissionsView } from '../views/permissions';
import { loadProofLedgerView } from '../views/proof_ledger';
import { loadSlicerStudioView } from '../views/slicer_studio';
import { loadStepsView } from '../views/steps';

export type AppRoute =
  | 'permissions'
  | 'capture'
  | 'evidence'
  | 'steps'
  | 'anchors'
  | 'slicer_studio'
  | 'proof_ledger'
  | 'model_dock'
  | 'agent_plant';

export interface CoreFlowRequest {
  session_label: string;
  output_dir: string;
  on_stage?: (stage: FlowStage) => void;
}

export type FlowStage =
  | 'session'
  | 'capture'
  | 'ocr'
  | 'steps'
  | 'tutorial'
  | 'validate'
  | 'export'
  | 'verify'
  | 'complete';

export interface CoreFlowResult {
  session_id: string;
  tutorial_export_ok: boolean;
  output_path: string;
  bundle_hash: string;
  verify_valid: boolean;
  verify_issues: string[];
}

export function createAppShell() {
  const store = createUiStore();

  async function loadRoute(route: AppRoute): Promise<unknown> {
    store.setActiveRoute(route);
    const activeSessionId = store.getState().active_session_id;

    switch (route) {
      case 'permissions':
        return loadPermissionsView();
      case 'capture':
        return loadCaptureView(activeSessionId);
      case 'evidence':
        return loadEvidenceView(activeSessionId);
      case 'steps':
        return loadStepsView(activeSessionId);
      case 'anchors':
        return loadAnchorsView(activeSessionId);
      case 'slicer_studio':
        return loadSlicerStudioView(activeSessionId);
      case 'proof_ledger':
        return loadProofLedgerView(activeSessionId);
      case 'model_dock':
        return loadModelDockView();
      case 'agent_plant':
        return loadAgentPlantView(activeSessionId);
      default:
        return loadPermissionsView();
    }
  }

  async function runCaptureToTutorialFlow(input: CoreFlowRequest): Promise<AppResult<CoreFlowResult>> {
    input.on_stage?.('session');
    const session = await ipc.session_create({ label: input.session_label, metadata: {} });
    if (!session.ok) {
      store.setLastError(session.error.code);
      return { ok: false, error: session.error };
    }

    const sessionId = session.value.session_id;
    store.setActiveSession(sessionId);

    input.on_stage?.('capture');
    const captureStart = await ipc.capture_start({
      session_id: sessionId,
    });
    if (!captureStart.ok) {
      store.setLastError(captureStart.error.code);
      return { ok: false, error: captureStart.error };
    }

    input.on_stage?.('ocr');
    const ocr = await ipc.ocr_schedule({ session_id: sessionId });
    if (!ocr.ok) {
      store.setLastError(ocr.error.code);
      return { ok: false, error: ocr.error };
    }

    input.on_stage?.('steps');
    const steps = await ipc.steps_generate_candidates({
      session_id: sessionId,
    });
    if (!steps.ok) {
      store.setLastError(steps.error.code);
      return { ok: false, error: steps.error };
    }

    input.on_stage?.('tutorial');
    const tutorial = await ipc.tutorial_generate({
      session_id: sessionId,
    });
    if (!tutorial.ok) {
      store.setLastError(tutorial.error.code);
      return { ok: false, error: tutorial.error };
    }

    input.on_stage?.('validate');
    const validate = await ipc.tutorial_validate_export({ session_id: sessionId });
    if (!validate.ok) {
      store.setLastError(validate.error.code);
      return { ok: false, error: validate.error };
    }

    if (!validate.value.allowed) {
      return {
        ok: false,
        error: {
          code: 'EXPORT_GATE_FAILED',
          message: validate.value.reasons.join('; '),
          recoverable: false,
          action_hint: 'Fix steps or anchors before export',
        },
      };
    }

    input.on_stage?.('export');
    const exported = await ipc.tutorial_export_pack({ session_id: sessionId, output_dir: input.output_dir });
    if (!exported.ok) {
      store.setLastError(exported.error.code);
      return { ok: false, error: exported.error };
    }

    input.on_stage?.('verify');
    const verify = await ipc.export_verify_bundle({
      bundle_path: exported.value.output_path,
    });
    if (!verify.ok) {
      store.setLastError(verify.error.code);
      return { ok: false, error: verify.error };
    }

    if (!verify.value.valid) {
      return {
        ok: false,
        error: {
          code: 'EXPORT_GATE_FAILED',
          message: 'Export verification failed',
          details: verify.value.issues.join('; '),
          recoverable: true,
          action_hint: 'Inspect proof ledger and fix policy issues',
        },
      };
    }

    const _ = await ipc.capture_stop({ session_id: sessionId });

    store.setLastError(undefined);
    input.on_stage?.('complete');
    return {
      ok: true,
      value: {
        session_id: sessionId,
        tutorial_export_ok: true,
        output_path: exported.value.output_path,
        bundle_hash: exported.value.bundle_hash,
        verify_valid: verify.value.valid,
        verify_issues: verify.value.issues,
      },
    };
  }

  return {
    store,
    loadRoute,
    runCaptureToTutorialFlow,
  };
}
