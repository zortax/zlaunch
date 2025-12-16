//! Calculator module for evaluating mathematical expressions.
//!
//! This module provides functionality to:
//! - Detect if user input looks like a calculator expression
//! - Evaluate expressions using fasteval

mod detection;
mod evaluation;

pub use detection::looks_like_expression;
pub use evaluation::{CalcResult, evaluate_expression};
