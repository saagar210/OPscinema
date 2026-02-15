import { ipc } from '../../ipc/typed';

export interface EvidenceViewModel {
  coverage_pct: number;
  missing_step_ids: string[];
  evidence_count: number;
}

export async function loadEvidenceView(sessionId: string | undefined): Promise<EvidenceViewModel> {
  if (!sessionId) {
    return {
      coverage_pct: 0,
      missing_step_ids: [],
      evidence_count: 0,
    };
  }

  const coverage = await ipc.evidence_get_coverage({
    session_id: sessionId,
  });

  const set = await ipc.evidence_for_time_range({
    session_id: sessionId,
    start_ms: 0,
    end_ms: Number.MAX_SAFE_INTEGER,
  });

  return {
    coverage_pct: coverage.ok
      ? coverage.value.pass
        ? 100
        : 0
      : 0,
    missing_step_ids: coverage.ok ? coverage.value.missing_step_ids : [],
    evidence_count: set.ok ? set.value.evidence.length : 0,
  };
}
