//! Expression evaluation using fend.
//!
//! Wraps fend to provide a simple interface for evaluating
//! mathematical expressions and formatting results.

use crate::items::CalculatorItem;
use fend_core::Context;
use std::sync::{Mutex, OnceLock};

static CONTEXT: OnceLock<Mutex<Context>> = OnceLock::new();

/// Evaluate a mathematical expression.
///
/// Returns `Ok(CalculatorItem)` if the expression can be parsed,
/// or `None` if parsing fails entirely.
pub fn evaluate_expression(input: &str) -> Result<CalculatorItem, String> {
    let expression = input.trim().to_string();

    let mut context = CONTEXT
        .get_or_init(|| Mutex::new(Context::new()))
        .lock()
        .unwrap();
    match fend_core::evaluate(&expression, &mut context) {
        Ok(value) => {
            let value = value.get_main_result();
            let calc_value = value.trim_start_matches("approx. ");
            Ok(CalculatorItem {
                id: "calculator-result".to_string(),
                expression,
                display_result: format_display(value),
                clipboard_result: Some(calc_value.to_string()),
                is_error: false,
            })
        }
        Err(err) => {
            if err == "division by zero" {
                Ok(CalculatorItem {
                    id: "calculator-result".to_string(),
                    expression,
                    display_result: "Infinity".to_string(),
                    clipboard_result: None,
                    is_error: true,
                })
            } else {
                Err(err.to_string())
            }
        }
    }
}

/// Format a number for display with thousand separators.
fn format_display(value: &str) -> String {
    // Convert to f64, else return the original string
    let Ok(value) = value.parse::<f64>() else {
        return value.to_string();
    };
    if value.fract() == 0.0 && value.abs() < 1e15 {
        // Integer display with thousand separators
        format_with_separators(value as i64)
    } else {
        // Decimal display
        let formatted = format!("{:.10}", value);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');

        // Add thousand separators to the integer part
        if let Some(dot_pos) = trimmed.find('.') {
            let (int_part, dec_part) = trimmed.split_at(dot_pos);
            let int_val: i64 = int_part.parse().unwrap_or(0);
            format!("{}{}", format_with_separators(int_val), dec_part)
        } else {
            let int_val: i64 = trimmed.parse().unwrap_or(0);
            format_with_separators(int_val)
        }
    }
}

/// Format an integer with thousand separators.
fn format_with_separators(value: i64) -> String {
    let is_negative = value < 0;
    let abs_value = value.abs();
    let s = abs_value.to_string();

    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    let formatted: String = result.chars().rev().collect();
    if is_negative {
        format!("-{}", formatted)
    } else {
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate_expression;

    #[test]
    fn test_basic_evaluation() {
        let result = evaluate_expression("2 + 2").unwrap();
        assert_eq!(result.display_result, "4");
        assert_eq!(result.text_for_clipboard(), "4");
    }

    #[test]
    fn test_thousand_separators() {
        let result = evaluate_expression("1000 * 1000").unwrap();
        assert_eq!(result.display_result, "1,000,000");
        assert_eq!(result.text_for_clipboard(), "1000000");
    }

    #[test]
    fn test_decimal_result() {
        let result = evaluate_expression("1 / 3").unwrap();
        // Should have decimal places, no trailing zeros
        assert!(result.display_result.starts_with("approx. 0.333"));
    }

    #[test]
    fn test_division_by_zero() {
        let result = evaluate_expression("1 / 0").unwrap();
        assert_eq!(result.display_result, "Infinity");
    }

    #[test]
    fn test_invalid_expression() {
        // Truly invalid expressions that fasteval cannot parse
        let result = evaluate_expression("2 +* 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_functions() {
        // Use exponentiation for square root since sqrt is not built-in
        let result = evaluate_expression("16^0.5").unwrap();
        assert_eq!(result.display_result, "4");
    }

    #[test]
    fn test_trig_functions() {
        let result = evaluate_expression("sin(0)").unwrap();
        assert_eq!(result.display_result, "0");
    }
}
