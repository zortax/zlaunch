//! Expression evaluation using fend.
//!
//! Wraps fend to provide a simple interface for evaluating
//! mathematical expressions and formatting results.

use crate::items::CalculatorItem;
use fend_core::{Context, FendResult, SpanKind};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Interrupt handler that stops fend evaluation after a timeout.
struct TimeoutInterrupt {
    deadline: Instant,
}

impl TimeoutInterrupt {
    fn new(timeout: Duration) -> Self {
        Self {
            deadline: Instant::now() + timeout,
        }
    }
}

impl fend_core::Interrupt for TimeoutInterrupt {
    fn should_interrupt(&self) -> bool {
        Instant::now() >= self.deadline
    }
}

static CONTEXT: OnceLock<Mutex<Context>> = OnceLock::new();

/// Evaluate a mathematical expression.
///
/// Returns `Ok(CalculatorItem)` if the expression can be parsed,
/// or `Err` if parsing fails entirely.
pub fn evaluate_expression(input: &str) -> Result<CalculatorItem, String> {
    let expression = input.trim().to_string();

    let mut context = CONTEXT
        .get_or_init(|| Mutex::new(Context::new()))
        .lock()
        .unwrap();
    let interrupt = TimeoutInterrupt::new(Duration::from_millis(50));
    match fend_core::evaluate_with_interrupt(&expression, &mut context, &interrupt) {
        Ok(value) => {
            let calc_value = value.get_main_result().trim_start_matches("approx. ");
            Ok(CalculatorItem {
                id: "calculator-result".to_string(),
                expression,
                display_result: format_display(&value),
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
fn format_display(value: &FendResult) -> String {
    let mut output = String::new();

    for span in value.get_main_result_spans() {
        match span.kind() {
            SpanKind::Number => format_with_separators(span.string(), &mut output),
            _ => output.push_str(span.string()),
        }
    }

    output
}

/// Format an integer with thousand separators.
fn format_with_separators(value: &str, result: &mut String) {
    // Check if the value is a complex number or other unsupported kind
    for c in value.chars() {
        match c {
            'A'..='F' | 'a'..='f' | '0'..='9' | '.' => {}
            _ => {
                result.push_str(value);
                return;
            }
        }
    }

    let (integer, fraction) = value
        .split_once('.')
        .map(|(int, frac)| (int, Some(frac)))
        .unwrap_or((value, None));

    let (digits, negative) = integer
        .strip_prefix('-')
        .map(|s| (s, true))
        .unwrap_or((integer, false));

    result.reserve(value.len() + (digits.len() - 1) / 3);

    if negative {
        result.push('-');
    }

    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    if let Some(frac) = fraction {
        result.push('.');
        result.push_str(frac);
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
        assert!(result.display_result.starts_with("approx. 0.3333333333"));
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
