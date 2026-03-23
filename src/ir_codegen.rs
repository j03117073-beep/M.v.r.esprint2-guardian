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

//! IR to Deterministic Rust Code Generation
//!
//! Converts IR modules into deterministic Rust modules that can be executed
//! inside the sovereign kernel.

use crate::universal_frontend::{IRModule, IRFunction, IRNode, Value, BinaryOperator, UnaryOperator};
use std::collections::HashMap;

/// Input for IR execution
#[derive(Debug, Clone)]
pub struct IRInput {
    pub args: HashMap<String, Value>,
}

/// Result of IR execution
#[derive(Debug, Clone)]
pub struct IRResult {
    pub value: Value,
    pub bus_messages: Vec<crate::sovereign_bus::SovereignMessage>,
}

/// Generate deterministic Rust code from IR module
pub fn generate_rust_code(ir_module: &IRModule) -> String {
    let mut code = String::new();

    // Add necessary imports
    code.push_str("#![deny(unsafe_code)]

");
    code.push_str("use std::collections::HashMap;
");
    code.push_str("use crate::universal_frontend::{Value, IRInput, IRResult};
");
    code.push_str("use crate::sovereign_bus::SovereignMessage;

");

    // Generate constants
    for (name, value) in &ir_module.constants {
        let value_str = match value {
            Value::Int(i) => format!("Value::Int({})", i),
            Value::Float(f) => format!("Value::Float({})", f),
            Value::Bool(b) => format!("Value::Bool({})", b),
            Value::String(s) => format!("Value::String(\"{}\".to_string())", s),
        };
        code.push_str(&format!("const {}: Value = {};
", name.to_uppercase(), value_str));
    }
    code.push_str("
");

    // Generate functions
    for func in &ir_module.functions {
        code.push_str(&generate_function(func));
        code.push_str("
");
    }

    // Generate main execute function
    code.push_str("pub fn execute(input: IRInput) -> IRResult {
");
    if let Some(main_func) = ir_module.functions.first() {
        code.push_str(&format!("    {}(&input)
", main_func.name));
    } else {
        code.push_str("    IRResult { value: Value::Int(0), bus_messages: vec![] }
");
    }
    code.push_str("}
");

    code
}

/// Generate a single function
fn generate_function(func: &IRFunction) -> String {
    let mut code = format!("fn {}(input: &IRInput) -> IRResult {{
", func.name);

    // Declare local variables
    code.push_str("    let mut locals: HashMap<String, Value> = HashMap::new();
");

    // Add input arguments to locals
    for param in &func.parameters {
        code.push_str(&format!("    if let Some(val) = input.args.get(\"{}\") {{
", param));
        code.push_str(&format!("        locals.insert(\"{}\".to_string(), val.clone());
", param));
        code.push_str("    }
");
    }

    // Generate function body
    let (body_code, _) = generate_node(&func.body, 1);
    code.push_str(&body_code);

    code.push_str("    IRResult { value: Value::Int(0), bus_messages: vec![] }
"); // Default return
    code.push_str("}
");

    code
}

/// Generate code for an IR node, returns (code, variable_name)
fn generate_node(node: &IRNode, indent_level: usize) -> (String, String) {
    let indent = "    ".repeat(indent_level);
    match node {
        IRNode::Constant(val) => {
            let val_str = match val {
                Value::Int(i) => format!("Value::Int({})", i),
                Value::Float(f) => format!("Value::Float({})", f),
                Value::Bool(b) => format!("Value::Bool({})", b),
                Value::String(s) => format!("Value::String(\"{}\".to_string())", s),
            };
            (format!("{}let temp = {};
", indent, val_str), "temp".to_string())
        }
        IRNode::Variable(name) => {
            (String::new(), format!("locals.get(\"{}\").cloned().unwrap_or(Value::Int(0))", name))
        }
        IRNode::BinaryOp(left, op, right) => {
            let (left_code, left_var) = generate_node(left, indent_level);
            let (right_code, right_var) = generate_node(right, indent_level);
            let op_str = match op {
                BinaryOperator::Add => "+",
                BinaryOperator::Subtract => "-",
                BinaryOperator::Multiply => "*",
                BinaryOperator::Divide => "/",
                BinaryOperator::Equal => "==",
                BinaryOperator::NotEqual => "!=",
                BinaryOperator::Less => "<",
                BinaryOperator::LessEqual => "<=",
                BinaryOperator::Greater => ">",
                BinaryOperator::GreaterEqual => ">=",
                BinaryOperator::And => "&&",
                BinaryOperator::Or => "||",
            };
            let code = format!("{}{}{}let temp = {} {} {};
", left_code, right_code, indent, left_var, op_str, right_var);
            (code, "temp".to_string())
        }
        IRNode::UnaryOp(op, expr) => {
            let (expr_code, expr_var) = generate_node(expr, indent_level);
            let op_str = match op {
                UnaryOperator::Not => "!",
                UnaryOperator::Negate => "-",
            };
            let code = format!("{}{}let temp = {}{};
", expr_code, indent, op_str, expr_var);
            (code, "temp".to_string())
        }
        IRNode::Call(func_name, args) => {
            // Simplified - assume built-in functions
            let mut arg_vars = Vec::new();
            let mut code = String::new();
            for arg in args {
                let (arg_code, arg_var) = generate_node(arg, indent_level);
                code.push_str(&arg_code);
                arg_vars.push(arg_var);
            }
            let args_str = arg_vars.join(", ");
            let code_full = format!("{}{}let temp = {}_builtin({});
", code, indent, func_name, args_str);
            (code_full, "temp".to_string())
        }
        IRNode::If(cond, then_branch, else_branch) => {
            let (cond_code, cond_var) = generate_node(cond, indent_level);
            let (then_code, _) = generate_node(then_branch, indent_level + 1);
            let (else_code, _) = generate_node(else_branch, indent_level + 1);
            let code = format!("{}{}if {} {{
{}{}{}}} else {{
{}{}{}}}
",
                cond_code, indent, cond_var, then_code, indent, "}",
                else_code, indent, "}");
            (code, "Value::Int(0)".to_string()) // Placeholder
        }
        IRNode::Loop(_var, cond, body) => {
            // Bounded loop - assume condition eventually becomes false
            let (cond_code, cond_var) = generate_node(cond, indent_level);
            let (body_code, _) = generate_node(body, indent_level + 1);
            let code = format!("{}{}while {} {{
{}{}{}}}
",
                cond_code, indent, cond_var, body_code, indent, "}");
            (code, "Value::Int(0)".to_string()) // Placeholder
        }
        IRNode::Assign(var, expr) => {
            let (expr_code, expr_var) = generate_node(expr, indent_level);
            let code = format!("{}{}locals.insert(\"{}\".to_string(), {});
", expr_code, indent, var, expr_var);
            (code, var.clone())
        }
        IRNode::Return(expr) => {
            let (expr_code, expr_var) = generate_node(expr, indent_level);
            let code = format!("{}{}return IRResult {{ value: {}, bus_messages: vec![] }};
", expr_code, indent, expr_var);
            (code, "Value::Int(0)".to_string())
        }
        IRNode::Block(nodes) => {
            let mut code = String::new();
            for node in nodes {
                let (node_code, _) = generate_node(node, indent_level);
                code.push_str(&node_code);
            }
            (code, "Value::Int(0)".to_string())
        }
    }
}
