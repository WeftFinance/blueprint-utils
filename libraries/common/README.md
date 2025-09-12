# common

Utilities and helpers for building Scrypto blueprints. This crate provides reusable modules, traits, and macros to simplify configuration management, metadata initialization, service gating, and numeric helpers. A `prelude` is included for ergonomic imports.

## Features
- Configuration manager with versioning and history tracking
- Service management with role-aware updates and discovery macros
- Fluent metadata initialization for components and resources
- Validation helpers for `Decimal` and maps of `Decimal`
- Utility functions for Radix-specific checks and rounding
- Test and error helper macros

## Modules
- `modules::config_manager`: Versioned configuration storage with history, expiration, and the `Updatable` trait. Includes `updatable_config!` macro to bridge with `GenerateConfig` derive.
- `modules::service_manager`: Service status container and `ServiceManager` with Admin/Moderator roles. Includes `generate_service_variants!` macro.
- `modules::metadata_setter`: `MetadataSetter` trait to set and optionally lock metadata during initialization for components and resources.
- `utils::{functions,macros,traits,types}`: Small helpers, validation traits, and macros like `define_error!`, `create_event!`, and testing helpers.
- `prelude`: Re-exports common items plus `generate_config::GenerateConfig` and `safe_arith::safe_arith`.

## Install
Add the crate to your workspace or as a path dependency. Example (workspace root `Cargo.toml`):

```toml
[workspace.dependencies]
common = { path = "libraries/common" }
```

## Quick Start
Import the prelude for convenient access:

```rust
use common::prelude::*;
```

### Service management
Define your service enum and manage per-entity service states:

```rust
use common::prelude::*;

generate_service_variants!(
  pub enum MarketService,
  (ScryptoSbor, ManifestSbor, Debug, Clone, Copy, PartialEq, Eq, Hash),
  CreateCDP, UpdateCDP, BurnCDP
);

let mut status = ServiceStatus::<MarketService>::new();
assert!(status.check(&MarketService::CreateCDP));
status.set_status(MarketService::CreateCDP, false, StatusChangeType::ModeratorSet)?;
assert!(!status.check(&MarketService::CreateCDP));
```

Manage services across entities:

```rust
let store: KeyValueStore<u32, ServiceStatus<MarketService>> = KeyValueStore::new();
let mut manager = ServiceManager::new(store);
manager.set_entry(1)?; // initialize entity 1
manager.update(1, MarketService::CreateCDP, true, StatusChangeType::AdminSetAndLock)?;
manager.assert(1, &MarketService::CreateCDP)?;
```

Recommended role usage:
- Derive the authority (Admin/Moderator) from the caller’s badge/role inside your blueprint.
- Do not let external callers supply `StatusChangeType` — compute it server-side.
- For a safer baseline, initialize with `ServiceStatus::with_default(false)` and explicitly enable services.

### Configuration manager + GenerateConfig
Use `GenerateConfig` to derive an update enum and validation; then make it `Updatable` with `updatable_config!`:

```rust
use common::prelude::*;

#[derive(ScryptoSbor, ManifestSbor, Debug, Clone, PartialEq, Eq, Hash, GenerateConfig)]
struct TestConfig {
  #[check = "val.is_positive()"]
  amount: Decimal,
  #[check = "val.is_a_rate()"]
  rate: Decimal,
}

updatable_config!(TestConfig);

let mut cfg = TestConfig { amount: Decimal::ONE, rate: dec!("0.5") };
cfg.check()?;
let updates = indexset![
  UpdateTestConfigInput::Amount(Decimal::from(10)),
  UpdateTestConfigInput::Rate(dec!("0.8")),
];
cfg.update(updates)?;
```

Reference safety (optional):
- When binding external entities (e.g., CDPs) to a specific config version, call `inc_ref(key, version)`.
- When unbinding, call `dec_ref(key, version)`.
- Use `get_history_entry_strict(key, version)` to fetch only if still valid; it errors with `E_CFG_EXPIRED` instead of silently falling back.

### Metadata during initialization
Set and optionally lock metadata on components/resources during init:

```rust
use common::prelude::*;

let globalizing: Globalizing<MyBlueprint> = /* builder chain */ todo!();
let globalized = globalizing.set_init_metadata(vec![
  ("name".to_string(), MetadataValue::String("Weft".into()), true),
  ("description".to_string(), MetadataValue::String("Example".into()), false),
]).globalize();
```

### Utilities
- `checked_truncate(PreciseDecimal) -> anyhow::Result<Decimal>`
- `checked_round(Decimal, inner_precision) -> anyhow::Result<Decimal>`
- `check_lsu(ResourceAddress) -> Option<ComponentAddress>`
- `check_claim_nft(ResourceAddress) -> Option<ComponentAddress>`
- `check_recallable_resource(ResourceManager)` ensures recall roles are deny-all

## Notes
- This crate targets Scrypto 1.3.
- Prefer importing via `common::prelude::*` for macro and trait availability.
