use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, BinOp, Expr, ExprBinary, LitStr, Token};

/// Input structure for the macro that can handle both expression and optional error message
struct SafeArithmInput {
  expr: Expr,
  error_msg: Option<LitStr>,
}

impl Parse for SafeArithmInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let expr = input.parse()?;

    let error_msg = if input.peek(Token![,]) {
      input.parse::<Token![,]>()?;
      Some(input.parse()?)
    } else {
      None
    };

    Ok(SafeArithmInput { expr, error_msg })
  }
}

/// Converts arithmetic operations to checked methods that return Result
///
/// Example:
/// - safe_arithmetic!(a + c/b) -> a.checked_add(c.checked_div(b).ok_or("Arithmetic operation failed")?).ok_or("Arithmetic operation failed")
/// - safe_arithmetic!(a + c/b, "Overflow in calculation") -> a.checked_add(c.checked_div(b).ok_or("Overflow in calculation")?).ok_or("Overflow in calculation")
#[proc_macro]
pub fn safe_arithmetic(input: TokenStream) -> TokenStream {
  let SafeArithmInput { expr, error_msg } = parse_macro_input!(input as SafeArithmInput);

  let result = convert_to_checked_result(expr, &error_msg);
  quote! { #result }.into()
}

fn convert_to_checked_result(expr: Expr, error_msg: &Option<LitStr>) -> Expr {
  const DEFAULT_ERROR_MSG: &str = "Arithmetic operation failed";

  match expr {
    Expr::Binary(ExprBinary { left, op, right, .. }) => {
      let left_checked = convert_to_checked_result(*left, error_msg);
      let right_checked = convert_to_checked_result(*right, error_msg);

      let method_name = match op {
        BinOp::Add(_) => "checked_add",
        BinOp::Sub(_) => "checked_sub",
        BinOp::Mul(_) => "checked_mul",
        BinOp::Div(_) => "checked_div",
        BinOp::Rem(_) => "checked_rem",
        _ => panic!("Unsupported binary operation"),
      };

      let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());

      // Create the checked method call that returns Result
      let err_msg = error_msg.as_ref().map(|msg| msg.value()).unwrap_or(DEFAULT_ERROR_MSG.to_string());

      syn::parse_quote! {
          match (#left_checked, #right_checked) {
              (Ok(left_val), Ok(right_val)) => {
                  left_val.#method_ident(right_val).ok_or(#err_msg)
              }
              (Err(e), _) => Err(e),
              (_, Err(e)) => Err(e),
          }
      }
    }
    Expr::Paren(paren) => {
      // Handle parentheses by recursively converting the inner expression
      convert_to_checked_result(*paren.expr, error_msg)
    }
    // For non-binary expressions (variables, literals, etc.), wrap in Ok
    _ => {
      syn::parse_quote! {
          Ok(#expr)
      }
    }
  }
}

#[cfg(test)]
mod tests {

  #[test]
  fn test_simple_addition() {
    // This would be tested in an integration test with actual macro expansion
    // Here we just verify the macro compiles
  }
}

// Example usage (this would be in a separate crate that uses the macro):
/*
use safe_arithmetic::safe_arithmetic;

fn main() -> Result<(), &'static str> {
    let a = 10i32;
    let b = 2i32;
    let c = 6i32;

    // Without error message (uses default error message)
    let result = safe_arithmetic!(a + c / b)?;
    println!("Result: {}", result); // Should print 13

    // With custom error message
    let result2 = safe_arithmetic!(a + c / b, "Arithmetic overflow occurred")?;
    println!("Result2: {}", result2); // Should print 13

    // More examples with error messages:
    let result3 = safe_arithmetic!(a - b * c, "Multiplication or subtraction overflow")?;
    let result4 = safe_arithmetic!((a + b) * c, "Addition or multiplication overflow")?;

    println!("Result3: {}", result3); // Should print -2
    println!("Result4: {}", result4); // Should print 36

    // Example that would return an error:
    // let overflow_result = safe_arithmetic!(i32::MAX + 1, "Integer overflow detected");
    // This would return Err("Integer overflow detected")

    // Division by zero example:
    // let zero = 0i32;
    // let div_by_zero = safe_arithmetic!(a / zero, "Division by zero error");
    // This would return Err("Division by zero error")

    Ok(())
}
*/
