# safe_arith

A proc-macro `safe_arith!` that transforms arithmetic expressions into chains of checked operations returning `Result`. This helps prevent overflow/underflow at runtime with clean, composable code.

## Supported
- Binary ops: `+`, `-`, `*`, `/`, `%` mapped to `checked_add/sub/mul/div/rem`
- Unary negation: `-x` via `checked_neg`
- Parentheses and grouping
- Optional custom error message

## Return type
The expanded code returns `Result<T, &'static str>` where `T` is the numeric type of the expression. Use `?` in functions that return a compatible `Result`, or map errors as needed.

## Examples

Basic usage:

```rust
use common::prelude::*; // re-exports safe_arith

let a = Decimal::from(10);
let b = Decimal::from(2);
let c = Decimal::from(5);

let result: Result<Decimal, &'static str> = safe_arith!(a + c / b);
assert_eq!(result.unwrap(), dec!(12));
```

With custom error message:

```rust
fn compute(a: Decimal, b: Decimal) -> anyhow::Result<Decimal> {
  // Propagate as anyhow::Error
  let x = safe_arith!(a / b, "division failed").map_err(anyhow::Error::msg)?;
  Ok(x)
}
```

Nested and short-circuiting:

```rust
// Only evaluates the right side if the left succeeded
let res = safe_arith!((a - b) * (c / b));
```

## Install
As a workspace/path dependency (proc-macro crate):

```toml
[workspace.dependencies]
safe_arith = { path = "libraries/safe_arith" }
```
