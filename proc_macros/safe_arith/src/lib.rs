use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, BinOp, Expr, ExprBinary, ExprGroup, ExprParen, ExprUnary, LitStr, Token, UnOp};

/// Input structure for the macro that can handle both expression and optional error message
struct SafeArithmeticInput {
  expr: Expr,
  error_msg: Option<LitStr>,
}

impl Parse for SafeArithmeticInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let expr = input.parse()?;

    let error_msg = if input.peek(Token![,]) {
      input.parse::<Token![,]>()?;
      Some(input.parse()?)
    } else {
      None
    };

    Ok(SafeArithmeticInput { expr, error_msg })
  }
}

/// Converts Arithmetic operations to checked methods that return Result
///
/// Example:
/// - safe_arith!(a + c/b) -> a.checked_add(c.checked_div(b).ok_or("Arithmetic operation failed")?).ok_or("Arithmetic operation failed")
/// - safe_arith!(a + c/b, "Overflow in calculation") -> a.checked_add(c.checked_div(b).ok_or("Overflow in calculation")?).ok_or("Overflow in calculation")
#[proc_macro]
pub fn safe_arith(input: TokenStream) -> TokenStream {
  let SafeArithmeticInput { expr, error_msg } = parse_macro_input!(input as SafeArithmeticInput);

  let result = convert_to_checked_result(expr, &error_msg);
  quote! { #result }.into()
}

fn convert_to_checked_result(expr: Expr, error_msg: &Option<LitStr>) -> Expr {
  // Prefer embedding a string literal as an expression so we don't allocate here
  let err_expr: Expr = if let Some(msg) = error_msg {
    syn::parse_quote! { #msg }
  } else {
    syn::parse_quote! { "Arithmetic operation failed" }
  };

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
        _ => {
          // Friendlier compile-time error at call site
          return syn::parse_quote! {
            compile_error!("safe_arith: unsupported binary operation")
          };
        }
      };

      let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());

      // Short-circuit: only eval right if left succeeded
      syn::parse_quote! {
        match #left_checked {
          Ok(left_val) => match #right_checked {
            Ok(right_val) => left_val.#method_ident(right_val).ok_or(#err_expr),
            Err(e) => Err(e),
          },
          Err(e) => Err(e),
        }
      }
    }
    Expr::Unary(ExprUnary { op: UnOp::Neg(_), expr, .. }) => {
      let inner_checked = convert_to_checked_result(*expr, error_msg);
      syn::parse_quote! {
        match #inner_checked {
          Ok(val) => val.checked_neg().ok_or(#err_expr),
          Err(e) => Err(e),
        }
      }
    }
    // Handle parentheses and grouped tokens by recursively converting the inner expression
    Expr::Paren(ExprParen { expr, .. }) => convert_to_checked_result(*expr, error_msg),
    Expr::Group(ExprGroup { expr, .. }) => convert_to_checked_result(*expr, error_msg),
    // For non-binary expressions (variables, literals, etc.), wrap in Ok
    _ => {
      syn::parse_quote! { Ok(#expr) }
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn compiles() {
    // expansion is tested in downstream crates
    // assert!(true);
  }
}
