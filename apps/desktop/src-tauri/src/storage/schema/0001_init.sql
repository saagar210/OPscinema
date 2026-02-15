CREATE TABLE IF NOT EXISTS sessions (
  session_id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  created_at TEXT NOT NULL,
  closed_at TEXT,
  head_seq INTEGER NOT NULL DEFAULT 0,
  head_hash TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS events (
  session_id TEXT NOT NULL,
  seq INTEGER NOT NULL,
  event_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  payload_canon_json TEXT NOT NULL,
  prev_event_hash TEXT,
  event_hash TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (session_id, seq),
  UNIQUE(event_id)
);

CREATE TABLE IF NOT EXISTS assets (
  asset_id TEXT PRIMARY KEY,
  rel_path TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS jobs (
  job_id TEXT PRIMARY KEY,
  job_type TEXT NOT NULL,
  session_id TEXT,
  status TEXT NOT NULL,
  progress_json TEXT,
  error_json TEXT,
  created_at TEXT NOT NULL,
  started_at TEXT,
  ended_at TEXT
);

CREATE TABLE IF NOT EXISTS ocr_blocks (
  session_id TEXT NOT NULL,
  frame_event_id TEXT NOT NULL,
  ocr_block_id TEXT NOT NULL,
  frame_ms INTEGER NOT NULL,
  text TEXT NOT NULL,
  bbox_json TEXT NOT NULL,
  confidence INTEGER NOT NULL,
  language TEXT,
  PRIMARY KEY (session_id, frame_event_id, ocr_block_id)
);

CREATE TABLE IF NOT EXISTS steps_snapshot (
  session_id TEXT PRIMARY KEY,
  head_seq INTEGER NOT NULL,
  steps_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS anchors_snapshot (
  session_id TEXT PRIMARY KEY,
  head_seq INTEGER NOT NULL,
  anchors_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS verifiers (
  verifier_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  spec_json TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS verifier_runs (
  run_id TEXT PRIMARY KEY,
  verifier_id TEXT NOT NULL,
  session_id TEXT NOT NULL,
  status TEXT NOT NULL,
  result_asset_id TEXT NOT NULL,
  logs_asset_id TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS exports (
  export_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  bundle_type TEXT NOT NULL,
  output_path TEXT NOT NULL,
  manifest_asset_id TEXT NOT NULL,
  bundle_hash TEXT NOT NULL,
  warnings_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS models (
  model_id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  label TEXT NOT NULL,
  digest TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS model_roles (
  id INTEGER PRIMARY KEY CHECK(id=1),
  tutorial_generation TEXT,
  screen_explainer TEXT,
  anchor_grounding TEXT
);

CREATE TABLE IF NOT EXISTS benchmarks (
  bench_id TEXT PRIMARY KEY,
  model_id TEXT NOT NULL,
  score INTEGER NOT NULL,
  created_at TEXT NOT NULL
);
