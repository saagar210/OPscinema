import { ipc } from '../../ipc/typed';

export interface ProofLedgerViewModel {
  warning_count: number;
  step_count: number;
  evidence_count: number;
  exports: Array<{
    export_id: string;
    output_path: string;
    bundle_hash: string;
    warnings: number;
    verify_valid?: boolean;
    verify_issues?: string[];
  }>;
}

export async function loadProofLedgerView(sessionId: string | undefined): Promise<ProofLedgerViewModel> {
  if (!sessionId) {
    return {
      warning_count: 0,
      step_count: 0,
      evidence_count: 0,
      exports: [],
    };
  }

  const proof = await ipc.proof_get_view({
    session_id: sessionId,
  });

  if (!proof.ok) {
    return {
      warning_count: 0,
      step_count: 0,
      evidence_count: 0,
      exports: [],
    };
  }

  const exportsListed = await ipc.exports_list({ session_id: sessionId });
  const exportRows = exportsListed.ok ? exportsListed.value.exports.slice(0, 5) : [];
  const exportsWithVerify = await Promise.all(
    exportRows.map(async (item) => {
      const verify = await ipc.export_verify_bundle({ bundle_path: item.output_path });
      return {
        export_id: item.export_id,
        output_path: item.output_path,
        bundle_hash: item.bundle_hash,
        warnings: item.warnings.length,
        verify_valid: verify.ok ? verify.value.valid : undefined,
        verify_issues: verify.ok ? verify.value.issues : undefined,
      };
    }),
  );

  return {
    warning_count: proof.value.warnings.length,
    step_count: proof.value.steps.length,
    evidence_count: proof.value.evidence.evidence.length,
    exports: exportsWithVerify,
  };
}
