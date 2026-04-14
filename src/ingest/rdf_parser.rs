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

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::collections::BTreeMap;
use std::io::BufRead;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedDec9(pub i128);

impl FixedDec9 {
    pub const SCALE: i128 = 1_000_000_000;

    pub fn from_str(raw: &str) -> Option<Self> {
        let s = raw.trim();
        if s.is_empty() {
            return None;
        }

        let mut chars = s.chars().peekable();
        let negative = matches!(chars.peek(), Some('-'));
        if negative {
            chars.next();
        }

        let mut int_part: i128 = 0;
        let mut frac_part: i128 = 0;
        let mut frac_digits = 0usize;
        let mut seen_dot = false;
        let mut seen_digit = false;

        for ch in chars {
            if ch == '.' {
                if seen_dot {
                    return None;
                }
                seen_dot = true;
                continue;
            }
            if !ch.is_ascii_digit() {
                return None;
            }
            seen_digit = true;
            let digit = (ch as u8 - b'0') as i128;
            if !seen_dot {
                int_part = int_part.checked_mul(10)?.checked_add(digit)?;
            } else if frac_digits < 9 {
                frac_part = frac_part.checked_mul(10)?.checked_add(digit)?;
                frac_digits += 1;
            }
        }

        if !seen_digit {
            return None;
        }

        while frac_digits < 9 {
            frac_part = frac_part.checked_mul(10)?;
            frac_digits += 1;
        }

        let mut combined = int_part.checked_mul(Self::SCALE)?.checked_add(frac_part)?;
        if negative {
            combined = -combined;
        }
        Some(Self(combined))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentKind {
    Breaker,
    Disconnector,
    Switch,
    AcLineSegment,
    PowerTransformer,
    SeriesCompensator,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConductingEquipment {
    pub id: String,
    pub kind: EquipmentKind,
    pub open: Option<bool>,
    pub normal_open: Option<bool>,
    pub r: Option<FixedDec9>,
    pub x: Option<FixedDec9>,
    pub bch: Option<FixedDec9>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terminal {
    pub id: String,
    pub conducting_equipment_id: String,
    pub connectivity_node_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectivityNode {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CimModel {
    pub nodes: BTreeMap<String, ConnectivityNode>,
    pub terminals: BTreeMap<String, Terminal>,
    pub equipment: BTreeMap<String, ConductingEquipment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RdfParseError {
    Xml(String),
    Utf8(String),
}

pub fn parse_cim_rdf<R: BufRead>(input: R) -> Result<CimModel, RdfParseError> {
    let mut reader = Reader::from_reader(input);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut model = CimModel::default();
    let mut active: Option<ActiveElement> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                on_start(&e, &mut active, &mut model)?;
            }
            Ok(Event::Empty(e)) => {
                on_empty(&e, &mut active, &mut model)?;
            }
            Ok(Event::Text(t)) => {
                if let Some(active_el) = active.as_mut() {
                    let raw = std::str::from_utf8(t.as_ref())
                        .map_err(|e| RdfParseError::Utf8(e.to_string()))?;
                    active_el.last_text = Some(xml_unescape(raw));
                }
            }
            Ok(Event::End(e)) => {
                on_end(&e, &mut active, &mut model)?;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(RdfParseError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok(model)
}

#[derive(Debug, Clone)]
struct ActiveElement {
    tag_name: String,
    object_id: String,
    kind: ActiveKind,
    last_child_tag: Option<String>,
    last_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveKind {
    ConnectivityNode,
    Terminal,
    Equipment(EquipmentKind),
    Other,
}

fn on_start(
    e: &BytesStart<'_>,
    active: &mut Option<ActiveElement>,
    model: &mut CimModel,
) -> Result<(), RdfParseError> {
    let tag = local_name(e.name().as_ref());

    if let Some((kind, object_id)) = classify_root_object(&tag, e)? {
        *active = Some(ActiveElement {
            tag_name: tag,
            object_id: object_id.clone(),
            kind,
            last_child_tag: None,
            last_text: None,
        });
        seed_object(kind, &object_id, model);
        return Ok(());
    }

    if let Some(a) = active.as_mut() {
        a.last_child_tag = Some(tag);
        a.last_text = None;
    }
    Ok(())
}

fn on_empty(
    e: &BytesStart<'_>,
    active: &mut Option<ActiveElement>,
    model: &mut CimModel,
) -> Result<(), RdfParseError> {
    let tag = local_name(e.name().as_ref());

    if let Some((kind, object_id)) = classify_root_object(&tag, e)? {
        seed_object(kind, &object_id, model);
        return Ok(());
    }

    if let Some(a) = active.as_mut() {
        if let Some(resource) = attr_value(e, "resource")? {
            apply_child_value(
                a.kind,
                &a.object_id,
                &tag,
                &normalize_ref(&resource),
                model,
            );
        }
    }

    Ok(())
}

fn on_end(
    e: &quick_xml::events::BytesEnd<'_>,
    active: &mut Option<ActiveElement>,
    model: &mut CimModel,
) -> Result<(), RdfParseError> {
    let tag = local_name(e.name().as_ref());
    if let Some(a) = active.as_mut() {
        if a.tag_name == tag {
            *active = None;
            return Ok(());
        }
        if let Some(child) = &a.last_child_tag {
            if *child == tag {
                if let Some(text) = a.last_text.take() {
                    apply_child_value(a.kind, &a.object_id, child, &text, model);
                }
                a.last_child_tag = None;
            }
        }
    }
    Ok(())
}

fn seed_object(kind: ActiveKind, id: &str, model: &mut CimModel) {
    match kind {
        ActiveKind::ConnectivityNode => {
            model
                .nodes
                .entry(id.to_string())
                .or_insert_with(|| ConnectivityNode { id: id.to_string() });
        }
        ActiveKind::Terminal => {
            model.terminals.entry(id.to_string()).or_insert_with(|| Terminal {
                id: id.to_string(),
                conducting_equipment_id: String::new(),
                connectivity_node_id: String::new(),
            });
        }
        ActiveKind::Equipment(eq_kind) => {
            model
                .equipment
                .entry(id.to_string())
                .or_insert_with(|| ConductingEquipment {
                    id: id.to_string(),
                    kind: eq_kind,
                    open: None,
                    normal_open: None,
                    r: None,
                    x: None,
                    bch: None,
                });
        }
        ActiveKind::Other => {}
    }
}

fn apply_child_value(
    kind: ActiveKind,
    object_id: &str,
    child_tag: &str,
    raw_value: &str,
    model: &mut CimModel,
) {
    match kind {
        ActiveKind::Terminal => {
            if let Some(t) = model.terminals.get_mut(object_id) {
                if child_tag.ends_with(".ConductingEquipment") {
                    t.conducting_equipment_id = normalize_ref(raw_value);
                } else if child_tag.ends_with(".ConnectivityNode") {
                    t.connectivity_node_id = normalize_ref(raw_value);
                }
            }
        }
        ActiveKind::Equipment(_) => {
            if let Some(eq) = model.equipment.get_mut(object_id) {
                if child_tag.ends_with(".open") {
                    eq.open = parse_bool(raw_value);
                } else if child_tag.ends_with(".normalOpen") {
                    eq.normal_open = parse_bool(raw_value);
                } else if child_tag.ends_with(".r") {
                    eq.r = FixedDec9::from_str(raw_value);
                } else if child_tag.ends_with(".x") {
                    eq.x = FixedDec9::from_str(raw_value);
                } else if child_tag.ends_with(".bch") {
                    eq.bch = FixedDec9::from_str(raw_value);
                }
            }
        }
        _ => {}
    }
}

fn classify_root_object(
    tag: &str,
    e: &BytesStart<'_>,
) -> Result<Option<(ActiveKind, String)>, RdfParseError> {
    let id = attr_value(e, "ID")?
        .or_else(|| attr_value(e, "about").ok().flatten())
        .map(|v| normalize_ref(&v));

    let Some(object_id) = id else {
        return Ok(None);
    };

    let kind = match tag {
        "ConnectivityNode" => ActiveKind::ConnectivityNode,
        "Terminal" => ActiveKind::Terminal,
        "Breaker" => ActiveKind::Equipment(EquipmentKind::Breaker),
        "Disconnector" => ActiveKind::Equipment(EquipmentKind::Disconnector),
        "Switch" => ActiveKind::Equipment(EquipmentKind::Switch),
        "ACLineSegment" => ActiveKind::Equipment(EquipmentKind::AcLineSegment),
        "PowerTransformer" => ActiveKind::Equipment(EquipmentKind::PowerTransformer),
        "SeriesCompensator" => ActiveKind::Equipment(EquipmentKind::SeriesCompensator),
        _ => ActiveKind::Other,
    };

    Ok(Some((kind, object_id)))
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn local_name(name: &[u8]) -> String {
    let s = String::from_utf8_lossy(name);
    match s.rsplit_once(':') {
        Some((_, right)) => right.to_string(),
        None => s.to_string(),
    }
}

fn attr_value(e: &BytesStart<'_>, needle_suffix: &str) -> Result<Option<String>, RdfParseError> {
    for attr in e.attributes().flatten() {
        let k = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        if k.ends_with(needle_suffix) {
            let value = std::str::from_utf8(attr.value.as_ref())
                .map_err(|er| RdfParseError::Utf8(er.to_string()))?;
            return Ok(Some(value.to_string()));
        }
    }
    Ok(None)
}

fn normalize_ref(raw: &str) -> String {
    raw.trim()
        .trim_start_matches('#')
        .trim_start_matches("urn:uuid:")
        .to_string()
}

fn xml_unescape(raw: &str) -> String {
    raw.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_switch_line_topology() {
        let xml = r##"
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:cim="http://iec.ch/TC57/2013/CIM-schema-cim16#">
  <cim:ConnectivityNode rdf:ID="CN_A"/>
  <cim:ConnectivityNode rdf:ID="CN_B"/>
  <cim:Breaker rdf:ID="BRK_1">
    <cim:Switch.normalOpen>false</cim:Switch.normalOpen>
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
    <cim:ACLineSegment.bch>0.001</cim:ACLineSegment.bch>
  </cim:ACLineSegment>
</rdf:RDF>"##;

        let parsed = parse_cim_rdf(xml.as_bytes()).expect("parses");
        assert_eq!(parsed.nodes.len(), 2);
        assert_eq!(parsed.terminals.len(), 2);
        assert!(parsed.equipment.contains_key("BRK_1"));
        assert!(parsed.equipment.contains_key("L1"));
        assert_eq!(parsed.equipment["L1"].r, Some(FixedDec9(10_000_000)));
        assert_eq!(parsed.equipment["L1"].x, Some(FixedDec9(100_000_000)));
        assert_eq!(parsed.equipment["L1"].bch, Some(FixedDec9(1_000_000)));
    }
}

