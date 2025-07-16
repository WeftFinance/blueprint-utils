use safe_arithmetic::safe_arithmetic;

#[test]
fn test_simple_addition() -> Result<(), &'static str> {
  let a = 10i32;
  let b = 5i32;
  let result = safe_arithmetic!(a + b)?;
  assert_eq!(result, 15);
  Ok(())
}

#[test]
fn test_simple_subtraction() -> Result<(), &'static str> {
  let a = 10i32;
  let b = 3i32;
  let result = safe_arithmetic!(a - b)?;
  assert_eq!(result, 7);
  Ok(())
}

#[test]
fn test_simple_multiplication() -> Result<(), &'static str> {
  let a = 6i32;
  let b = 7i32;
  let result = safe_arithmetic!(a * b)?;
  assert_eq!(result, 42);
  Ok(())
}

#[test]
fn test_simple_division() -> Result<(), &'static str> {
  let a = 20i32;
  let b = 4i32;
  let result = safe_arithmetic!(a / b)?;
  assert_eq!(result, 5);
  Ok(())
}

#[test]
fn test_simple_remainder() -> Result<(), &'static str> {
  let a = 17i32;
  let b = 5i32;
  let result = safe_arithmetic!(a % b)?;
  assert_eq!(result, 2);
  Ok(())
}

#[test]
fn test_complex_expression() -> Result<(), &'static str> {
  let a = 10i32;
  let b = 2i32;
  let c = 6i32;
  let result = safe_arithmetic!(a + c / b)?;
  assert_eq!(result, 13);
  Ok(())
}

#[test]
fn test_complex_expression_with_subtraction() -> Result<(), &'static str> {
  let a = 10i32;
  let b = 2i32;
  let c = 6i32;
  let result = safe_arithmetic!(a - b * c)?;
  assert_eq!(result, -2);
  Ok(())
}

#[test]
fn test_parentheses() -> Result<(), &'static str> {
  let a = 10i32;
  let b = 2i32;
  let c = 6i32;
  let result = safe_arithmetic!((a + b) * c)?;
  assert_eq!(result, 72);
  Ok(())
}

#[test]
fn test_nested_parentheses() -> Result<(), &'static str> {
  let a = 10i32;
  let b = 2i32;
  let c = 3i32;
  let d = 4i32;
  let result = safe_arithmetic!((a + b) * (c - d))?;
  assert_eq!(result, -12);
  Ok(())
}

#[test]
fn test_with_custom_error_message() -> Result<(), &'static str> {
  let a = 10i32;
  let b = 5i32;
  let result = safe_arithmetic!(a + b, "Addition overflow")?;
  assert_eq!(result, 15);
  Ok(())
}

#[test]
fn test_division_with_custom_error_message() -> Result<(), &'static str> {
  let a = 20i32;
  let b = 4i32;
  let result = safe_arithmetic!(a / b, "Division error")?;
  assert_eq!(result, 5);
  Ok(())
}

#[test]
fn test_complex_expression_with_custom_error_message() -> Result<(), &'static str> {
  let a = 10i32;
  let b = 2i32;
  let c = 6i32;
  let result = safe_arithmetic!(a + c / b, "Arithmetic overflow occurred")?;
  assert_eq!(result, 13);
  Ok(())
}

#[test]
fn test_addition_overflow() {
  let a = i32::MAX;
  let b = 1i32;
  let result: Result<i32, &'static str> = safe_arithmetic!(a + b);
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Arithmetic operation failed");
}

#[test]
fn test_subtraction_underflow() {
  let a = i32::MIN;
  let b = 1i32;
  let result: Result<i32, &'static str> = safe_arithmetic!(a - b);
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Arithmetic operation failed");
}

#[test]
fn test_multiplication_overflow() {
  let a = i32::MAX;
  let b = 2i32;
  let result: Result<i32, &'static str> = safe_arithmetic!(a * b);
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Arithmetic operation failed");
}

#[test]
fn test_division_by_zero() {
  let a = 10i32;
  let b = 0i32;
  let result: Result<i32, &'static str> = safe_arithmetic!(a / b);
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Arithmetic operation failed");
}

#[test]
fn test_remainder_by_zero() {
  let a = 10i32;
  let b = 0i32;
  let result: Result<i32, &'static str> = safe_arithmetic!(a % b);
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Arithmetic operation failed");
}

#[test]
fn test_custom_error_message_on_overflow() {
  let a = i32::MAX;
  let b = 1i32;
  let result: Result<i32, &'static str> = safe_arithmetic!(a + b, "Integer overflow detected");
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Integer overflow detected");
}

#[test]
fn test_custom_error_message_on_division_by_zero() {
  let a = 10i32;
  let b = 0i32;
  let result: Result<i32, &'static str> = safe_arithmetic!(a / b, "Division by zero error");
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Division by zero error");
}

#[test]
fn test_custom_error_message_on_multiplication_overflow() {
  let a = i32::MAX;
  let b = 2i32;
  let result: Result<i32, &'static str> = safe_arithmetic!(a * b, "Multiplication overflow");
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Multiplication overflow");
}

#[test]
fn test_custom_error_message_on_subtraction_underflow() {
  let a = i32::MIN;
  let b = 1i32;
  let result: Result<i32, &'static str> = safe_arithmetic!(a - b, "Subtraction underflow");
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Subtraction underflow");
}

#[test]
fn test_edge_case_zero_values() -> Result<(), &'static str> {
  let a = 0i32;
  let b = 0i32;
  let result = safe_arithmetic!(a + b)?;
  assert_eq!(result, 0);
  Ok(())
}

#[test]
fn test_edge_case_negative_values() -> Result<(), &'static str> {
  let a = -10i32;
  let b = -5i32;
  let result = safe_arithmetic!(a + b)?;
  assert_eq!(result, -15);
  Ok(())
}

#[test]
fn test_edge_case_mixed_signs() -> Result<(), &'static str> {
  let a = 10i32;
  let b = -3i32;
  let result = safe_arithmetic!(a + b)?;
  assert_eq!(result, 7);
  Ok(())
}

#[test]
fn test_edge_case_max_values() -> Result<(), &'static str> {
  let a = i32::MAX;
  let b = 0i32;
  let result = safe_arithmetic!(a + b)?;
  assert_eq!(result, i32::MAX);
  Ok(())
}

#[test]
fn test_edge_case_min_values() -> Result<(), &'static str> {
  let a = i32::MIN;
  let b = 0i32;
  let result = safe_arithmetic!(a + b)?;
  assert_eq!(result, i32::MIN);
  Ok(())
}

#[test]
fn test_chain_operations() -> Result<(), &'static str> {
  let a = 10i32;
  let b = 2i32;
  let c = 3i32;
  let d = 4i32;
  let result = safe_arithmetic!(a + b - c * d)?;
  assert_eq!(result, 0);
  Ok(())
}

#[test]
fn test_different_integer_types_u32() -> Result<(), &'static str> {
  let a = 10u32;
  let b = 5u32;
  let result = safe_arithmetic!(a + b)?;
  assert_eq!(result, 15);
  Ok(())
}

#[test]
fn test_different_integer_types_i64() -> Result<(), &'static str> {
  let a = 10i64;
  let b = 5i64;
  let result = safe_arithmetic!(a * b)?;
  assert_eq!(result, 50);
  Ok(())
}

#[test]
fn test_different_integer_types_u64() -> Result<(), &'static str> {
  let a = 100u64;
  let b = 10u64;
  let result = safe_arithmetic!(a / b)?;
  assert_eq!(result, 10);
  Ok(())
}

#[test]
fn test_u32_overflow() {
  let a = u32::MAX;
  let b = 1u32;
  let result: Result<u32, &'static str> = safe_arithmetic!(a + b);
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Arithmetic operation failed");
}

#[test]
fn test_u32_underflow() {
  let a = 0u32;
  let b = 1u32;
  let result: Result<u32, &'static str> = safe_arithmetic!(a - b);
  assert!(result.is_err());
  assert_eq!(result.unwrap_err(), "Arithmetic operation failed");
}

#[test]
fn test_operator_precedence() -> Result<(), &'static str> {
  let a = 2i32;
  let b = 3i32;
  let c = 4i32;
  let result = safe_arithmetic!(a + b * c)?;
  assert_eq!(result, 14); // Should be 2 + (3 * 4) = 14
  Ok(())
}

#[test]
fn test_nested_complex_expression() -> Result<(), &'static str> {
  let a = 10i32;
  let b = 2i32;
  let c = 3i32;
  let d = 4i32;
  let e = 5i32;
  let result = safe_arithmetic!((a + b) * (c - d) + e)?;
  assert_eq!(result, -7); // (10 + 2) * (3 - 4) + 5 = 12 * -1 + 5 = -7
  Ok(())
}

#[test]
fn test_multiple_parentheses_levels() -> Result<(), &'static str> {
  let a = 2i32;
  let b = 3i32;
  let c = 4i32;
  let d = 5i32;
  let result = safe_arithmetic!((a + b) * (c + d))?;
  assert_eq!(result, 45); // (2 + 3) * (4 + 5) = 5 * 9 = 45
  Ok(())
}

#[test]
fn test_remainder_with_negative_numbers() -> Result<(), &'static str> {
  let a = -17i32;
  let b = 5i32;
  let result = safe_arithmetic!(a % b)?;
  assert_eq!(result, -2);
  Ok(())
}

#[test]
fn test_remainder_with_both_negative() -> Result<(), &'static str> {
  let a = -17i32;
  let b = -5i32;
  let result = safe_arithmetic!(a % b)?;
  assert_eq!(result, -2);
  Ok(())
}

#[test]
fn test_single_variable() -> Result<(), &'static str> {
  let a = 42i32;
  let result = safe_arithmetic!(a)?;
  assert_eq!(result, 42);
  Ok(())
}

#[test]
fn test_single_literal() -> Result<(), &'static str> {
  let result = safe_arithmetic!(42)?;
  assert_eq!(result, 42);
  Ok(())
}

#[test]
fn test_long_chain_operations() -> Result<(), &'static str> {
  let a = 1i32;
  let b = 2i32;
  let c = 3i32;
  let d = 4i32;
  let e = 5i32;
  let result = safe_arithmetic!(a + b + c + d + e)?;
  assert_eq!(result, 15);
  Ok(())
}

#[test]
fn test_mixed_operations_chain() -> Result<(), &'static str> {
  let a = 20i32;
  let b = 4i32;
  let c = 2i32;
  let d = 3i32;
  let result = safe_arithmetic!(a / b * c + d)?;
  assert_eq!(result, 13); // 20 / 4 * 2 + 3 = 5 * 2 + 3 = 13
  Ok(())
}
