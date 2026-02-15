use crate::exports::fs::write_file;
use crate::exports::manifest::build_manifest;
use crate::policy::export_gate::{tutorial_pack_gate, ExportGateInput};
use crate::util::canon_json::to_canonical_json;
use opscinema_export_manifest::{BundleType, ManifestWarning, ModelPin, PolicyAttestations};
use opscinema_types::{ExportResult, ExportWarning, Step};
use std::path::Path;
use uuid::Uuid;

pub struct TutorialPackBuildOptions {
    pub missing_evidence: Vec<String>,
    pub degraded_anchor_ids: Vec<String>,
    pub warnings: Vec<ExportWarning>,
    pub model_pins: Vec<ModelPin>,
    pub offline_policy_enforced: bool,
}

pub fn export_tutorial_pack(
    session_id: Uuid,
    steps: &[Step],
    options: TutorialPackBuildOptions,
    output_dir: &Path,
) -> anyhow::Result<ExportResult> {
    let TutorialPackBuildOptions {
        missing_evidence,
        degraded_anchor_ids,
        warnings,
        model_pins,
        offline_policy_enforced,
    } = options;
    let strict_passed =
        missing_evidence.is_empty() && degraded_anchor_ids.is_empty() && warnings.is_empty();
    tutorial_pack_gate(&ExportGateInput {
        steps: steps.to_vec(),
        missing_evidence: missing_evidence.clone(),
        degraded_anchor_ids: degraded_anchor_ids.clone(),
        warnings: warnings.clone(),
    })?;

    let tutorial_json = to_canonical_json(&serde_json::json!({
        "schema_version": 1,
        "steps": steps,
    }))?;
    write_file(&output_dir.join("tutorial.json"), tutorial_json.as_bytes())?;
    let player_html = build_player_html(steps, strict_passed, missing_evidence.len());
    write_file(
        &output_dir.join("player/index.html"),
        player_html.as_bytes(),
    )?;

    let manifest = build_manifest(
        output_dir,
        BundleType::TutorialPack,
        &session_id.to_string(),
        warnings
            .iter()
            .map(|w| ManifestWarning {
                code: w.code.clone(),
                message: w.message.clone(),
            })
            .collect(),
        PolicyAttestations {
            evidence_coverage_passed: missing_evidence.is_empty(),
            tutorial_strict_passed: strict_passed,
            offline_policy_enforced,
        },
        model_pins,
    )?;
    let manifest_json = to_canonical_json(&manifest)?;
    write_file(&output_dir.join("manifest.json"), manifest_json.as_bytes())?;

    Ok(ExportResult {
        export_id: Uuid::new_v4(),
        output_path: output_dir.display().to_string(),
        bundle_hash: manifest.bundle_hash,
        warnings,
    })
}

fn build_player_html(steps: &[Step], strict_passed: bool, missing_evidence_count: usize) -> String {
    let mut step_rows = String::new();
    for step in steps {
        let mut block_rows = String::new();
        for block in &step.body.blocks {
            let refs = if block.evidence_refs.is_empty() {
                "none".to_string()
            } else {
                block
                    .evidence_refs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            block_rows.push_str(&format!(
                "<li><p class=\"block-text\">{}</p><p class=\"evidence\">Evidence: {}</p></li>",
                escape_html(&block.text),
                escape_html(&refs)
            ));
        }
        step_rows.push_str(&format!(
            "<article class=\"step\"><h2>{}. {}</h2><ul>{}</ul></article>",
            step.order_index + 1,
            escape_html(&step.title),
            block_rows
        ));
    }

    let strict_label = if strict_passed { "PASS" } else { "BLOCKED" };
    let strict_class = if strict_passed {
        "badge badge-pass"
    } else {
        "badge badge-block"
    };

    format!(
        "<!doctype html>\
<html lang=\"en\">\
<head>\
  <meta charset=\"utf-8\" />\
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\
  <title>OpsCinema Tutorial Pack</title>\
  <style>\
    :root {{ --bg:#f7f7f7; --ink:#111; --muted:#666; --card:#fff; --line:#ddd; --ok:#1f7a1f; --bad:#9c1a1a; }}\
    body {{ font-family: 'Avenir Next', 'Segoe UI', sans-serif; margin:0; background:var(--bg); color:var(--ink); }}\
    main {{ max-width: 980px; margin: 0 auto; padding: 24px; }}\
    .hero {{ background: linear-gradient(135deg, #ffffff, #eef5ff); border:1px solid var(--line); border-radius:14px; padding:18px 20px; margin-bottom:18px; }}\
    h1 {{ margin:0 0 8px; font-size: 28px; }}\
    .meta {{ color:var(--muted); margin: 0; font-size: 14px; }}\
    .badge {{ display:inline-block; padding:5px 10px; border-radius:999px; font-weight:700; font-size:12px; margin-right:8px; }}\
    .badge-pass {{ background:#e8f7e8; color:var(--ok); border:1px solid #b8e0b8; }}\
    .badge-block {{ background:#fdecec; color:var(--bad); border:1px solid #f4bcbc; }}\
    .step {{ background:var(--card); border:1px solid var(--line); border-radius:12px; padding:14px 16px; margin-bottom:10px; }}\
    .step h2 {{ margin:0 0 8px; font-size:20px; }}\
    .step ul {{ margin:0; padding-left:20px; }}\
    .block-text {{ margin:0 0 4px; }}\
    .evidence {{ margin:0 0 8px; color:var(--muted); font-size:13px; word-break:break-word; }}\
  </style>\
</head>\
<body>\
  <main>\
    <section class=\"hero\">\
      <h1>OpsCinema Tutorial Pack</h1>\
      <p class=\"meta\">Follow these steps to complete the handoff flow. Each generated block includes evidence references.</p>\
      <p><span class=\"{strict_class}\">Tutorial strict: {strict_label}</span><span class=\"badge\">Missing evidence refs: {missing_evidence_count}</span></p>\
    </section>\
    {step_rows}\
  </main>\
</body>\
</html>"
    )
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#39;")
}
