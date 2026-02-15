import assert from 'node:assert/strict';

import { createAppShell } from '../src/app/index';
import { setIpcRuntimeInvoke } from '../src/ipc/client';
import type { AppError, IpcCommand } from '../src/ipc/generated';
import { applyStepEditWithConflictRetry } from '../src/views/steps';

type AppResult<T> = { ok: true; value: T } | { ok: false; error: AppError };

const sessionId = 'session-ui-test';
let nextJob = 0;
let headSeq = 1;
let shouldConflict = true;

function ok<T>(value: T): AppResult<T> {
  return { ok: true, value };
}

setIpcRuntimeInvoke(async (command: IpcCommand, payload: unknown) => {
  switch (command) {
    case 'session_create':
      return ok({
        session_id: sessionId,
        label: 'ui',
        created_at: new Date().toISOString(),
        head_seq: 0,
        head_hash: 'h',
      });
    case 'capture_start':
      return ok({ state: 'CAPTURING', session_id: sessionId, started_at: new Date().toISOString() });
    case 'ocr_schedule':
    case 'steps_generate_candidates':
    case 'tutorial_generate':
      nextJob += 1;
      return ok({ job_id: `job-${nextJob}` });
    case 'tutorial_validate_export':
      return ok({ allowed: true, reasons: [] });
    case 'tutorial_export_pack':
      return ok({
        export_id: 'export-1',
        output_path: '/tmp/export',
        bundle_hash: 'hash',
        warnings: [],
      });
    case 'export_verify_bundle':
      return ok({
        valid: true,
        issues: [],
      });
    case 'steps_list':
      return ok({
        steps: [
          {
            step_id: 'step-1',
            order_index: 0,
            title: 'First',
            body: { blocks: [] },
            risk_tags: [],
          },
        ],
        head_seq: headSeq,
      });
    case 'steps_apply_edit':
      if (shouldConflict) {
        shouldConflict = false;
        return {
          ok: false,
          error: {
            code: 'CONFLICT',
            message: 'conflict',
            recoverable: true,
          },
        };
      }
      headSeq += 1;
      return ok({ head_seq: headSeq, applied: true });
    default:
      return ok(payload ?? {});
  }
});

async function main(): Promise<void> {
  const shell = createAppShell();
  const flow = await shell.runCaptureToTutorialFlow({
    session_label: 'integration',
    output_dir: '/tmp/ui-integration',
  });
  assert.equal(flow.ok, true);
  if (flow.ok) {
    assert.equal(flow.value.session_id, sessionId);
    assert.equal(flow.value.tutorial_export_ok, true);
  }

  const retried = await applyStepEditWithConflictRetry(sessionId, 1, {
    update_title: { step_id: 'step-1', title: 'Retried' },
  });
  assert.equal(retried.ok, true);
  if (retried.ok) {
    assert.equal(retried.value.applied, true);
    assert.equal(retried.value.head_seq, 2);
  }

  console.log('UI integration flow passed');
}

void main();
