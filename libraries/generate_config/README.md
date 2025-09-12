# generate_config

A procedural macro that derives update and validation logic for configuration structs. It generates an `Update{StructName}Input` enum and `impl` methods `update()` and `check()` based on your fields and optional `#[check]` attributes.

## What it generates
- Enum: `Update{StructName}Input` with one variant per field
- Method: `fn update(&mut self, IndexSet<Update...>) -> Result<(), String>`
- Method: `fn check(&self) -> Result<(), String>`

## Supported field kinds
- Simple fields: any type (e.g. `Decimal`, `String`, `u32`, addresses)
- Sets: `BTreeSet<T>`, `HashSet<T>`, `IndexSet<T>` using a single variant per field with `(T, bool)` payload: `true` = add, `false` = remove
- Maps: `BTreeMap<K,V>`, `HashMap<K,V>`, `IndexMap<K,V>` using `Variant(K, Option<V>)` to insert/update or remove

## Field validation
Attach `#[check = "<expr>"]` to a field to run custom validation during `check()`. The expression is parsed into a closure `|val: &T| -> bool` and executed; any `false` produces `Err("Invalid {Struct}::{field}")`.

Examples:

```rust
#[derive(GenerateConfig)]
struct ValidationConfig {
  #[check = "val.is_positive()"]
  amount: Decimal,
  #[check = "val.is_a_rate()"]
  percentage: Decimal,
}
```

## Semantics
- `update()` applies changes first, then calls `check()`.
- If validation fails, the in-memory struct remains mutated; persist only after `Ok(()))` or use a wrapper (e.g., `ConfigurationManager`) that validates before storing.

## Example

```rust
use common::prelude::*; // brings GenerateConfig and IndexSet

#[derive(Debug, GenerateConfig)]
pub struct TestConfig {
  pub valuator_component: ComponentAddress,
  pub valuator_method: String,
  pub is_enabled: bool,
  #[check = "val.is_a_rate()"]
  pub rate: Decimal,
  pub underlying_resources: BTreeSet<String>,
  pub resource_map: BTreeMap<String, Decimal>,
}

// Generated enum: UpdateTestConfigInput { ValuatorComponent(...), Rate(...), UnderlyingResources((T, bool)), ResourceMap(String, Option<Decimal>), ... }

let mut cfg = TestConfig {
  valuator_component: GENESIS_HELPER,
  valuator_method: "method".to_string(),
  is_enabled: true,
  rate: dec!(0.5),
  underlying_resources: BTreeSet::new(),
  resource_map: BTreeMap::new(),
};

let updates = indexset![
  UpdateTestConfigInput::ValuatorComponent(CONSENSUS_MANAGER),
  UpdateTestConfigInput::Rate(dec!(0.8)),
  UpdateTestConfigInput::UnderlyingResources(("resource1".to_string(), true)),
  UpdateTestConfigInput::ResourceMap("resource_key1".to_string(), Some(dec!(1.2))),
];

cfg.update(updates).unwrap();
assert_eq!(cfg.valuator_component, CONSENSUS_MANAGER);
assert!(cfg.underlying_resources.contains("resource1"));
assert_eq!(cfg.resource_map.get("resource_key1"), Some(&dec!(1.2)));
```

## Using with ConfigurationManager
The `common` crate includes `updatable_config!` to implement its `Updatable` trait by delegating to the generated `update()`/`check()` (converting `String` errors to `anyhow::Error`).

```rust
use common::prelude::*;

#[derive(ScryptoSbor, ManifestSbor, Debug, Clone, PartialEq, Eq, Hash, GenerateConfig)]
struct MyConfig { /* fields... */ }

updatable_config!(MyConfig);
```

## Install
As a workspace/path dependency (proc-macro crate):

```toml
[workspace.dependencies]
generate_config = { path = "libraries/generate_config" }
```

## Notes
- Targets Scrypto 1.3 and uses `IndexSet`/`IndexMap` via `scrypto::prelude::*`.
- Map updates use `(key, Some(value))` to insert/update and `(key, None)` to remove.
