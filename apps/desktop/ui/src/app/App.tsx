import { useEffect, useMemo, useState } from 'react';
import type { AppRoute, FlowStage } from './index';
import { createAppShell } from './index';
import { subscribeRuntimeEvent } from '../ipc/client';
import { ipc } from '../ipc/typed';
import { applyStepEditWithConflictRetry } from '../views/steps';
import type { PermissionsViewModel } from '../views/permissions';
import type { CaptureViewModel } from '../views/capture';
import type { EvidenceViewModel } from '../views/evidence';
import type { StepsViewModel } from '../views/steps';
import type { AnchorsViewModel } from '../views/anchors';
import type { SlicerStudioViewModel } from '../views/slicer_studio';
import type { ProofLedgerViewModel } from '../views/proof_ledger';
import type { ModelDockViewModel } from '../views/model_dock';
import type { AgentPlantViewModel } from '../views/agent_plant';

const ROUTES: AppRoute[] = [
  'permissions',
  'capture',
  'evidence',
  'steps',
  'anchors',
  'slicer_studio',
  'proof_ledger',
  'model_dock',
  'agent_plant',
];

const STAGES: FlowStage[] = [
  'session',
  'capture',
  'ocr',
  'steps',
  'tutorial',
  'validate',
  'export',
  'verify',
  'complete',
];

interface RuntimeCaptureEvent {
  stream_seq: number;
  sent_at: string;
  payload: {
    state: 'IDLE' | 'CAPTURING' | 'STOPPED';
    session_id?: string;
  };
}

interface RuntimeJobStatusEvent {
  stream_seq: number;
  sent_at: string;
  payload: {
    job_id: string;
    status: 'QUEUED' | 'RUNNING' | 'SUCCEEDED' | 'FAILED' | 'CANCELLED';
  };
}

interface RuntimeJobProgressEvent {
  stream_seq: number;
  sent_at: string;
  payload: {
    job_id: string;
    stage: string;
    pct: number;
    counters: {
      done: number;
      total: number;
    };
  };
}

interface CaptureTelemetry {
  event_count: number;
  last_seq: number | null;
  gap_count: number;
}

export function App(): JSX.Element {
  const shell = useMemo(() => createAppShell(), []);
  const [route, setRoute] = useState<AppRoute>('permissions');
  const [routeData, setRouteData] = useState<unknown>({});
  const [outputDir, setOutputDir] = useState('/tmp/opscinema-ui-export');
  const [flowStage, setFlowStage] = useState<FlowStage | 'idle'>('idle');
  const [lastMessage, setLastMessage] = useState('Ready');
  const [flowRunning, setFlowRunning] = useState(false);
  const [lastCaptureState, setLastCaptureState] = useState<'IDLE' | 'CAPTURING' | 'STOPPED'>('IDLE');
  const [lastJobStatus, setLastJobStatus] = useState<Record<string, RuntimeJobStatusEvent['payload']['status']>>({});
  const [lastJobProgress, setLastJobProgress] = useState<Record<string, RuntimeJobProgressEvent['payload']>>({});
  const [verifyIssues, setVerifyIssues] = useState<string[]>([]);
  const [captureTelemetry, setCaptureTelemetry] = useState<CaptureTelemetry>({
    event_count: 0,
    last_seq: null,
    gap_count: 0,
  });

  const activeSessionId = shell.store.getState().active_session_id;

  async function refreshRoute(nextRoute: AppRoute): Promise<void> {
    const data = await shell.loadRoute(nextRoute);
    setRouteData(data);
  }

  useEffect(() => {
    let cancelled = false;
    void shell.loadRoute(route).then((data) => {
      if (!cancelled) {
        setRouteData(data);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [route, shell]);

  useEffect(() => {
    let unlistenJobStatus: (() => void) | undefined;
    let unlistenCapture: (() => void) | undefined;
    let unlistenProgress: (() => void) | undefined;

    void subscribeRuntimeEvent('job_status', (payload) => {
      const event = payload as RuntimeJobStatusEvent;
      shell.store.ingestJobStatus(event as never);
      setLastJobStatus((prev) => ({ ...prev, [event.payload.job_id]: event.payload.status }));
      if (event.payload.status === 'FAILED') {
        setLastMessage(`Job ${event.payload.job_id} failed`);
        setFlowRunning(false);
      }
    }).then((unlisten) => {
      unlistenJobStatus = unlisten;
    });

    void subscribeRuntimeEvent('job_progress', (payload) => {
      const event = payload as RuntimeJobProgressEvent;
      setLastJobProgress((prev) => ({ ...prev, [event.payload.job_id]: event.payload }));
    }).then((unlisten) => {
      unlistenProgress = unlisten;
    });

    void subscribeRuntimeEvent('capture_status', (payload) => {
      const event = payload as RuntimeCaptureEvent;
      shell.store.ingestCaptureStatus(event as never);
      setLastCaptureState(event.payload.state);
      setLastMessage(`Capture ${event.payload.state.toLowerCase()}`);
      setCaptureTelemetry((prev) => {
        const skipped =
          prev.last_seq !== null && event.stream_seq > prev.last_seq + 1
            ? event.stream_seq - prev.last_seq - 1
            : 0;
        return {
          event_count: prev.event_count + 1,
          last_seq: event.stream_seq,
          gap_count: prev.gap_count + skipped,
        };
      });
    }).then((unlisten) => {
      unlistenCapture = unlisten;
    });

    return () => {
      if (unlistenJobStatus) unlistenJobStatus();
      if (unlistenProgress) unlistenProgress();
      if (unlistenCapture) unlistenCapture();
    };
  }, [shell]);

  async function runCoreFlow(): Promise<void> {
    setFlowRunning(true);
    setFlowStage('session');
    setVerifyIssues([]);
    setLastMessage('Starting handoff flow');

    const result = await shell.runCaptureToTutorialFlow({
      session_label: `handoff-${Date.now()}`,
      output_dir: outputDir,
      on_stage: (stage) => {
        setFlowStage(stage);
        setLastMessage(`Stage: ${stage}`);
      },
    });

    if (result.ok) {
      setFlowRunning(false);
      setFlowStage('complete');
      setLastMessage(`Export verified: ${result.value.output_path}`);
      setRoute('slicer_studio');
      await refreshRoute('slicer_studio');
      return;
    }

    setFlowRunning(false);
    setLastMessage(`Flow failed: ${result.error.code} ${result.error.message}`);
    if (result.error.details) {
      setVerifyIssues(result.error.details.split('; ').filter((value) => value.length > 0));
    }
  }

  async function runOptimisticStepEdit(): Promise<void> {
    if (!activeSessionId) {
      setLastMessage('No active session for step edit');
      return;
    }

    const listed = await ipc.steps_list({ session_id: activeSessionId });
    if (!listed.ok || listed.value.steps.length === 0) {
      setLastMessage('No steps available');
      return;
    }

    const first = listed.value.steps[0];
    const edit = await applyStepEditWithConflictRetry(activeSessionId, listed.value.head_seq, {
      update_title: {
        step_id: first.step_id,
        title: `${first.title} (handoff edit)`,
      },
    });

    if (edit.ok) {
      setLastMessage(`Step edit applied at sequence ${edit.value.head_seq}`);
      await refreshRoute('steps');
      return;
    }

    setLastMessage(`Step edit failed: ${edit.error.code}`);
  }

  const progressRows = Object.values(lastJobProgress).sort((a, b) => a.job_id.localeCompare(b.job_id));

  return (
    <div className="app-shell">
      <aside className="workflow-rail">
        <h1>OpsCinema Suite</h1>
        <p className="subtitle">Ops handoff capture, evidence, and verified tutorial export.</p>

        <div className="cta-card">
          <button className="primary-btn" disabled={flowRunning} onClick={() => void runCoreFlow()}>
            Start Handoff Session
          </button>
          <button className="secondary-btn" disabled={flowRunning} onClick={() => void runOptimisticStepEdit()}>
            Apply Step Edit (Retry)
          </button>
        </div>

        <label htmlFor="outputDir">Export directory</label>
        <input
          id="outputDir"
          value={outputDir}
          onChange={(event) => setOutputDir(event.currentTarget.value)}
          className="output-input"
        />

        <div className="status-grid">
          <div>
            <span className="meta-label">Session</span>
            <strong>{activeSessionId ?? 'none'}</strong>
          </div>
          <div>
            <span className="meta-label">Capture</span>
            <strong>{lastCaptureState}</strong>
          </div>
          <div>
            <span className="meta-label">Flow</span>
            <strong>{flowRunning ? 'running' : 'idle'}</strong>
          </div>
          <div>
            <span className="meta-label">Capture stream seq</span>
            <strong>{captureTelemetry.last_seq ?? 0}</strong>
          </div>
        </div>

        <div className="stage-list" role="list" aria-label="Flow stages">
          {STAGES.map((stage) => (
            <div
              key={stage}
              role="listitem"
              className={`stage-chip ${flowStage === stage ? 'active' : ''}`}
            >
              {stage}
            </div>
          ))}
        </div>

        <p className="status-line">{lastMessage}</p>

        {verifyIssues.length > 0 ? (
          <div className="error-box">
            <h3>Flow Issues</h3>
            <ul>
              {verifyIssues.map((issue) => (
                <li key={issue}>{issue}</li>
              ))}
            </ul>
          </div>
        ) : null}
      </aside>

      <main className="content-panel">
        <nav className="route-nav">
          {ROUTES.map((value) => (
            <button
              key={value}
              onClick={() => setRoute(value)}
              className={`route-btn ${route === value ? 'active' : ''}`}
              disabled={flowRunning}
            >
              {value}
            </button>
          ))}
        </nav>

        <section className="route-card">
          <header>
            <h2>{route.replace('_', ' ')}</h2>
          </header>
          <RoutePanel route={route} routeData={routeData} />
        </section>

        <section className="route-card">
          <header>
            <h2>Runtime Job Progress</h2>
          </header>
          <p>
            Capture events: {captureTelemetry.event_count} (sequence gaps observed: {captureTelemetry.gap_count})
          </p>
          {progressRows.length === 0 ? <p>No runtime job events received yet.</p> : null}
          {progressRows.map((row) => (
            <article key={row.job_id} className="job-row">
              <div className="job-row-head">
                <strong>{row.job_id}</strong>
                <span>{lastJobStatus[row.job_id] ?? 'QUEUED'}</span>
              </div>
              <p>
                Stage: {row.stage} ({row.counters.done}/{row.counters.total})
              </p>
              <progress max={100} value={row.pct} />
            </article>
          ))}
        </section>
      </main>
    </div>
  );
}

function RoutePanel(props: { route: AppRoute; routeData: unknown }): JSX.Element {
  switch (props.route) {
    case 'permissions':
      return <PermissionsPanel model={props.routeData as Partial<PermissionsViewModel>} />;
    case 'capture':
      return <CapturePanel model={props.routeData as Partial<CaptureViewModel>} />;
    case 'evidence':
      return <EvidencePanel model={props.routeData as Partial<EvidenceViewModel>} />;
    case 'steps':
      return <StepsPanel model={props.routeData as Partial<StepsViewModel>} />;
    case 'anchors':
      return <AnchorsPanel model={props.routeData as Partial<AnchorsViewModel>} />;
    case 'slicer_studio':
      return <SlicerPanel model={props.routeData as Partial<SlicerStudioViewModel>} />;
    case 'proof_ledger':
      return <ProofPanel model={props.routeData as Partial<ProofLedgerViewModel>} />;
    case 'model_dock':
      return <ModelPanel model={props.routeData as Partial<ModelDockViewModel>} />;
    case 'agent_plant':
      return <AgentPanel model={props.routeData as Partial<AgentPlantViewModel>} />;
    default:
      return <p>Unknown route</p>;
  }
}

function PermissionsPanel({ model }: { model: Partial<PermissionsViewModel> }): JSX.Element {
  return (
    <div className="metric-grid">
      <MetricCard label="Screen recording" value={model.screen_recording ? 'Granted' : 'Missing'} />
      <MetricCard label="Accessibility" value={model.accessibility ? 'Granted' : 'Missing'} />
      <MetricCard label="Full disk access" value={model.full_disk_access ? 'Granted' : 'Missing'} />
      <MetricCard label="Offline policy" value={model.offline_mode ? 'Enabled' : 'Disabled'} />
    </div>
  );
}

function CapturePanel({ model }: { model: Partial<CaptureViewModel> }): JSX.Element {
  const frames = model.keyframes ?? [];
  return (
    <div>
      <p>Capture status: <strong>{model.status?.state ?? 'UNKNOWN'}</strong></p>
      <p>Keyframes captured: <strong>{frames.length}</strong></p>
      <ul>
        {frames.slice(0, 8).map((frame) => (
          <li key={frame.frame_event_id}>
            {frame.frame_ms} ms - {frame.asset_id}
          </li>
        ))}
      </ul>
    </div>
  );
}

function EvidencePanel({ model }: { model: Partial<EvidenceViewModel> }): JSX.Element {
  const coveragePct = model.coverage_pct ?? 0;
  const missingStepIds = model.missing_step_ids ?? [];
  const evidenceCount = model.evidence_count ?? 0;
  return (
    <div>
      <p>Evidence coverage: <strong>{coveragePct}%</strong></p>
      <progress value={coveragePct} max={100} />
      <p>Evidence items: {evidenceCount}</p>
      {missingStepIds.length > 0 ? (
        <ul>
          {missingStepIds.map((stepId) => (
            <li key={stepId}>Missing refs for step {stepId}</li>
          ))}
        </ul>
      ) : (
        <p>No missing evidence refs detected.</p>
      )}
    </div>
  );
}

function StepsPanel({ model }: { model: Partial<StepsViewModel> }): JSX.Element {
  const steps = model.steps ?? [];
  return (
    <div>
      <p>Head sequence: {model.head_seq ?? 0}</p>
      <ol>
        {steps.map((step) => (
          <li key={step.step_id}>
            {step.title} <span className="muted">({step.step_id})</span>
          </li>
        ))}
      </ol>
    </div>
  );
}

function AnchorsPanel({ model }: { model: Partial<AnchorsViewModel> }): JSX.Element {
  const anchors = model.anchors ?? [];
  return (
    <div>
      <p>Step: {model.step_id ?? 'none selected'}</p>
      {anchors.length === 0 ? <p>No anchors yet.</p> : null}
      <ul>
        {anchors.map((anchor) => (
          <li key={anchor.anchor_id}>
            {anchor.anchor_id} - confidence {anchor.confidence.toFixed(2)} -{' '}
            {anchor.degraded ? 'degraded' : 'stable'}
          </li>
        ))}
      </ul>
    </div>
  );
}

function SlicerPanel({ model }: { model: Partial<SlicerStudioViewModel> }): JSX.Element {
  const issues = model.issues ?? [];
  return (
    <div>
      <p>
        Tutorial strict gate: <strong>{model.strict_ready ? 'ready' : 'blocked'}</strong>
      </p>
      {issues.length > 0 ? (
        <ul>
          {issues.map((issue) => (
            <li key={issue}>{issue}</li>
          ))}
        </ul>
      ) : (
        <p>No strictness issues detected.</p>
      )}
    </div>
  );
}

function ProofPanel({ model }: { model: Partial<ProofLedgerViewModel> }): JSX.Element {
  const exports = model.exports ?? [];
  return (
    <div>
      <div className="metric-grid">
        <MetricCard label="Warning count" value={String(model.warning_count ?? 0)} />
        <MetricCard label="Step count" value={String(model.step_count ?? 0)} />
        <MetricCard label="Evidence count" value={String(model.evidence_count ?? 0)} />
      </div>
      <h3>Recent exports</h3>
      {exports.length === 0 ? <p>No exports yet.</p> : null}
      <ul>
        {exports.map((value) => (
          <li key={value.export_id}>
            {value.export_id} - {value.verify_valid ? 'verified' : 'invalid'} - warnings{' '}
            {value.warnings}
          </li>
        ))}
      </ul>
    </div>
  );
}

function ModelPanel({ model }: { model: Partial<ModelDockViewModel> }): JSX.Element {
  const roleMap = model.role_map ?? {};
  return (
    <div>
      <div className="metric-grid">
        <MetricCard label="Registered models" value={String(model.model_count ?? 0)} />
        <MetricCard label="Bench records" value={String(model.bench_count ?? 0)} />
      </div>
      <h3>Role assignments</h3>
      <ul>
        {Object.entries(roleMap).map(([role, modelId]) => (
          <li key={role}>
            {role}: {modelId || 'unassigned'}
          </li>
        ))}
      </ul>
    </div>
  );
}

function AgentPanel({ model }: { model: Partial<AgentPlantViewModel> }): JSX.Element {
  const pipelines = model.pipelines ?? [];
  return (
    <div>
      <p>Can run pipeline: {model.can_run ? 'yes' : 'no active session'}</p>
      <ul>
        {pipelines.map((pipeline) => (
          <li key={pipeline}>{pipeline}</li>
        ))}
      </ul>
    </div>
  );
}

function MetricCard(props: { label: string; value: string }): JSX.Element {
  return (
    <article className="metric-card">
      <span className="meta-label">{props.label}</span>
      <strong>{props.value}</strong>
    </article>
  );
}
