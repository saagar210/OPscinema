use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct PipelineDag {
    pub nodes: BTreeSet<String>,
    pub edges: BTreeMap<String, Vec<String>>,
}

impl PipelineDag {
    pub fn topological(&self) -> Vec<String> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        for n in &self.nodes {
            visit(n, &self.edges, &mut seen, &mut out);
        }
        out
    }
}

fn visit(
    node: &str,
    edges: &BTreeMap<String, Vec<String>>,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<String>,
) {
    if !seen.insert(node.to_string()) {
        return;
    }
    if let Some(next) = edges.get(node) {
        for n in next {
            visit(n, edges, seen, out);
        }
    }
    out.push(node.to_string());
}
