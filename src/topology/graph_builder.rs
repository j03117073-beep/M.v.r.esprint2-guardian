// Copyright (c) 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// This file is part of the M.V.R.ESPRINT1 Sovereign Execution System,
// including TLBSS geometry, the Universal Execution Layer, the
// Deterministic IR, Rust Codegen Pipeline, SovereignBus, and the
// Cryptographic Audit Chain.
//
// No part of this file, its algorithms, structures, or designs may be
// copied, reproduced, modified, distributed, published, sublicensed,
// reverse-engineered, or used to create derivative works without the
// express written permission of OBINNA JAMES EJIOFOR.
//
// This software contains proprietary trade secrets and confidential
// intellectual property. Unauthorized use is strictly prohibited.
#![deny(unsafe_code)]

use crate::ingest::rdf_parser::{CimModel, EquipmentKind, FixedDec9, Terminal};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyDivergence {
    pub equipment_id: String,
    pub modeled_closed: bool,
    pub telemetered_closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectricalBus {
    pub bus_id: String,
    pub connectivity_nodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchEdge {
    pub equipment_id: String,
    pub kind: EquipmentKind,
    pub from_bus: String,
    pub to_bus: String,
    pub r: Option<FixedDec9>,
    pub x: Option<FixedDec9>,
    pub bch: Option<FixedDec9>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TopologyGraph {
    pub buses: Vec<ElectricalBus>,
    pub branches: Vec<BranchEdge>,
    pub divergences: Vec<TopologyDivergence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologyBuildMode {
    ModelState,
    TelemetryOverride,
}

pub fn build_topology_graph(
    model: &CimModel,
    telemetered_switch_closed: &BTreeMap<String, bool>,
) -> TopologyGraph {
    build_topology_graph_with_mode(model, telemetered_switch_closed, TopologyBuildMode::ModelState)
}

pub fn build_topology_graph_with_mode(
    model: &CimModel,
    telemetered_switch_closed: &BTreeMap<String, bool>,
    mode: TopologyBuildMode,
) -> TopologyGraph {
    let mut uf = UnionFind::new(model.nodes.keys().cloned().collect());
    let terminals_by_eq = terminals_by_equipment(&model.terminals);
    let mut divergences = Vec::new();

    for (eq_id, eq) in &model.equipment {
        if !is_switch_kind(eq.kind) {
            continue;
        }
        let modeled_closed = modeled_closed(eq.open, eq.normal_open);
        let effective_closed = if let Some(telem) = telemetered_switch_closed.get(eq_id) {
            if *telem != modeled_closed {
                divergences.push(TopologyDivergence {
                    equipment_id: eq_id.clone(),
                    modeled_closed,
                    telemetered_closed: *telem,
                });
            }
            if mode == TopologyBuildMode::TelemetryOverride {
                *telem
            } else {
                modeled_closed
            }
        } else {
            modeled_closed
        };
        if effective_closed {
            close_switch_nodes(eq_id, &terminals_by_eq, &mut uf);
        }
    }

    let buses = derive_buses(model, &mut uf);
    let node_to_bus = node_to_bus_lookup(&buses);
    let mut branches = Vec::new();

    for (eq_id, eq) in &model.equipment {
        if is_switch_kind(eq.kind) {
            continue;
        }
        let Some(terms) = terminals_by_eq.get(eq_id) else {
            continue;
        };
        if terms.len() < 2 {
            continue;
        }

        let mut mapped: Vec<&str> = terms
            .iter()
            .filter_map(|t| node_to_bus.get(&t.connectivity_node_id).map(|s| s.as_str()))
            .collect();
        mapped.sort_unstable();
        mapped.dedup();
        if mapped.len() < 2 {
            continue;
        }

        branches.push(BranchEdge {
            equipment_id: eq_id.clone(),
            kind: eq.kind,
            from_bus: mapped[0].to_string(),
            to_bus: mapped[1].to_string(),
            r: eq.r.clone(),
            x: eq.x.clone(),
            bch: eq.bch.clone(),
        });
    }

    branches.sort_by(|a, b| {
        a.equipment_id
            .cmp(&b.equipment_id)
            .then(a.from_bus.cmp(&b.from_bus))
            .then(a.to_bus.cmp(&b.to_bus))
    });
    divergences.sort_by(|a, b| a.equipment_id.cmp(&b.equipment_id));

    TopologyGraph {
        buses,
        branches,
        divergences,
    }
}

fn modeled_closed(open: Option<bool>, normal_open: Option<bool>) -> bool {
    match open {
        Some(v) => !v,
        None => !normal_open.unwrap_or(false),
    }
}

fn close_switch_nodes(
    eq_id: &str,
    terminals_by_eq: &BTreeMap<String, Vec<Terminal>>,
    uf: &mut UnionFind,
) {
    let Some(terms) = terminals_by_eq.get(eq_id) else {
        return;
    };
    let mut nodes: Vec<&str> = terms.iter().map(|t| t.connectivity_node_id.as_str()).collect();
    nodes.sort_unstable();
    nodes.dedup();
    if nodes.len() < 2 {
        return;
    }
    let anchor = nodes[0];
    for n in nodes.iter().skip(1) {
        uf.union(anchor, n);
    }
}

fn derive_buses(model: &CimModel, uf: &mut UnionFind) -> Vec<ElectricalBus> {
    let mut buckets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for node_id in model.nodes.keys() {
        let root = uf.find(node_id);
        buckets.entry(root).or_default().insert(node_id.clone());
    }

    let mut buses = Vec::new();
    for (_root, members) in buckets {
        let connectivity_nodes: Vec<String> = members.into_iter().collect();
        let bus_id = format!("BUS::{}", connectivity_nodes.join("+"));
        buses.push(ElectricalBus {
            bus_id,
            connectivity_nodes,
        });
    }
    buses.sort_by(|a, b| a.bus_id.cmp(&b.bus_id));
    buses
}

fn node_to_bus_lookup(buses: &[ElectricalBus]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for bus in buses {
        for node in &bus.connectivity_nodes {
            out.insert(node.clone(), bus.bus_id.clone());
        }
    }
    out
}

fn terminals_by_equipment(terminals: &BTreeMap<String, Terminal>) -> BTreeMap<String, Vec<Terminal>> {
    let mut grouped: BTreeMap<String, Vec<Terminal>> = BTreeMap::new();
    for t in terminals.values() {
        if t.conducting_equipment_id.is_empty() || t.connectivity_node_id.is_empty() {
            continue;
        }
        grouped
            .entry(t.conducting_equipment_id.clone())
            .or_default()
            .push(t.clone());
    }
    for values in grouped.values_mut() {
        values.sort_by(|a, b| a.id.cmp(&b.id));
    }
    grouped
}

fn is_switch_kind(kind: EquipmentKind) -> bool {
    matches!(
        kind,
        EquipmentKind::Breaker | EquipmentKind::Disconnector | EquipmentKind::Switch
    )
}

#[derive(Debug, Clone)]
struct UnionFind {
    parent: BTreeMap<String, String>,
}

impl UnionFind {
    fn new(nodes: Vec<String>) -> Self {
        let parent = nodes.iter().map(|n| (n.clone(), n.clone())).collect();
        Self { parent }
    }

    fn find(&mut self, x: &str) -> String {
        let p = self
            .parent
            .get(x)
            .cloned()
            .unwrap_or_else(|| x.to_string());
        if p == x {
            return p;
        }
        let root = self.find(&p);
        self.parent.insert(x.to_string(), root.clone());
        root
    }

    fn union(&mut self, a: &str, b: &str) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        let (parent, child) = if ra <= rb { (ra, rb) } else { (rb, ra) };
        self.parent.insert(child, parent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::rdf_parser::parse_cim_rdf;

    #[test]
    fn closed_breaker_collapses_nodes_and_detects_divergence() {
        let xml = r##"
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:cim="http://iec.ch/TC57/2013/CIM-schema-cim16#">
  <cim:ConnectivityNode rdf:ID="CN_A"/>
  <cim:ConnectivityNode rdf:ID="CN_B"/>
  <cim:ConnectivityNode rdf:ID="CN_C"/>
  <cim:Breaker rdf:ID="BRK_1">
    <cim:Switch.open>false</cim:Switch.open>
  </cim:Breaker>
  <cim:Terminal rdf:ID="T1">
    <cim:Terminal.ConductingEquipment rdf:resource="#BRK_1"/>
    <cim:Terminal.ConnectivityNode rdf:resource="#CN_A"/>
  </cim:Terminal>
  <cim:Terminal rdf:ID="T2">
    <cim:Terminal.ConductingEquipment rdf:resource="#BRK_1"/>
    <cim:Terminal.ConnectivityNode rdf:resource="#CN_B"/>
  </cim:Terminal>
  <cim:ACLineSegment rdf:ID="L1">
    <cim:ACLineSegment.r>0.01</cim:ACLineSegment.r>
    <cim:ACLineSegment.x>0.1</cim:ACLineSegment.x>
  </cim:ACLineSegment>
  <cim:Terminal rdf:ID="T3">
    <cim:Terminal.ConductingEquipment rdf:resource="#L1"/>
    <cim:Terminal.ConnectivityNode rdf:resource="#CN_B"/>
  </cim:Terminal>
  <cim:Terminal rdf:ID="T4">
    <cim:Terminal.ConductingEquipment rdf:resource="#L1"/>
    <cim:Terminal.ConnectivityNode rdf:resource="#CN_C"/>
  </cim:Terminal>
</rdf:RDF>"##;

        let model = parse_cim_rdf(xml.as_bytes()).expect("parse");
        let mut telem = BTreeMap::new();
        telem.insert("BRK_1".to_string(), false);
        let graph = build_topology_graph(&model, &telem);

        assert_eq!(graph.buses.len(), 2);
        assert_eq!(graph.branches.len(), 1);
        assert_eq!(graph.divergences.len(), 1);
        assert_eq!(graph.divergences[0].equipment_id, "BRK_1");
        assert!(graph.divergences[0].modeled_closed);
        assert!(!graph.divergences[0].telemetered_closed);
    }

    #[test]
    fn telemetry_override_splits_bus_when_breaker_is_open() {
        let xml = r##"
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:cim="http://iec.ch/TC57/2013/CIM-schema-cim16#">
  <cim:ConnectivityNode rdf:ID="CN_A"/>
  <cim:ConnectivityNode rdf:ID="CN_B"/>
  <cim:ConnectivityNode rdf:ID="CN_C"/>
  <cim:Breaker rdf:ID="BRK_1">
    <cim:Switch.open>false</cim:Switch.open>
  </cim:Breaker>
  <cim:Terminal rdf:ID="T1">
    <cim:Terminal.ConductingEquipment rdf:resource="#BRK_1"/>
    <cim:Terminal.ConnectivityNode rdf:resource="#CN_A"/>
  </cim:Terminal>
  <cim:Terminal rdf:ID="T2">
    <cim:Terminal.ConductingEquipment rdf:resource="#BRK_1"/>
    <cim:Terminal.ConnectivityNode rdf:resource="#CN_B"/>
  </cim:Terminal>
</rdf:RDF>"##;

        let model = parse_cim_rdf(xml.as_bytes()).expect("parse");
        let mut telem = BTreeMap::new();
        telem.insert("BRK_1".to_string(), false);

        let graph = build_topology_graph_with_mode(
            &model,
            &telem,
            TopologyBuildMode::TelemetryOverride,
        );
        assert_eq!(graph.buses.len(), 3);
        assert_eq!(graph.divergences.len(), 1);
    }
}

