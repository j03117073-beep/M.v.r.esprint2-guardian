// Copyright © 2026 OBINNA JAMES EJIOFOR
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

//! IR Backends — Output Re-emission Layer
//!
//! Converts Rust execution results back into the caller's language.
//! Enforces deterministic execution constraints via the DeterministicExecutable trait.

use crate::universal_frontend::{IRModule, Value};
use crate::ir_codegen::IRResult;
use crate::testament_audit::{ExecutionTrace, TraceEvent};
use crate::deterministic_core::{DetScheduler, DetTask};

// Use `DetScheduler` and `DetTask` from `deterministic_core` to centralize
// scheduler semantics and avoid ad-hoc map replacements across the codebase.

/// Trait for deterministic execution: all implementers must be tick-based, order-preserving
pub trait DeterministicExecutable {
    /// Execute deterministically: no implicit parallelism, no async scheduling
    fn execute_deterministic(
        &self,
        scheduler: &mut DetScheduler,
    ) -> Result<(IRResult, ExecutionTrace), String>;
}

/// Trait for language backends
pub trait LanguageBackend {
    fn emit(&self, result: &IRResult, original_ir: &IRModule) -> String;
}

/// Python backend
pub struct PythonBackend;

impl LanguageBackend for PythonBackend {
    fn emit(&self, result: &IRResult, _original_ir: &IRModule) -> String {
        match &result.value {
            Value::Int(i) => format!("return {}", i),
            Value::Float(f) => format!("return {}", f),
            Value::Bool(b) => format!("return {}", b),
            Value::String(s) => format!("return \"{}\"", s),
        }
    }
}

/// JavaScript backend
pub struct JavascriptBackend;

impl LanguageBackend for JavascriptBackend {
    fn emit(&self, result: &IRResult, _original_ir: &IRModule) -> String {
        match &result.value {
            Value::Int(i) => format!("return {};", i),
            Value::Float(f) => format!("return {};", f),
            Value::Bool(b) => format!("return {};", b),
            Value::String(s) => format!("return \"{}\";", s),
        }
    }
}

/// C# backend
pub struct CSharpBackend;

impl LanguageBackend for CSharpBackend {
    fn emit(&self, result: &IRResult, _original_ir: &IRModule) -> String {
        match &result.value {
            Value::Int(i) => format!("return {};", i),
            Value::Float(f) => format!("return {}f;", f),
            Value::Bool(b) => format!("return {};", b),
            Value::String(s) => format!("return \"{}\";", s),
        }
    }
}

/// Go backend
pub struct GoBackend;

impl LanguageBackend for GoBackend {
    fn emit(&self, result: &IRResult, _original_ir: &IRModule) -> String {
        match &result.value {
            Value::Int(i) => format!("return {}", i),
            Value::Float(f) => format!("return {}", f),
            Value::Bool(b) => format!("return {}", b),
            Value::String(s) => format!("return \"{}\"", s),
        }
    }
}
