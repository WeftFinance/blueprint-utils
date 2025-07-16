/*!
# Metadata Setter Module

This module provides a convenient trait for setting metadata on Scrypto components
and resources during their initialization phase. It offers a fluent interface that
integrates seamlessly with Scrypto's builder patterns.

## Key Features

- **Fluent Interface**: Chains naturally with existing Scrypto builders
- **Batch Operations**: Set multiple metadata entries in a single call
- **Lock Control**: Granular control over which metadata can be changed later
- **Type Safety**: Uses Scrypto's MetadataValue enum for compile-time type safety
- **Universal Support**: Works with both components and resources

## Supported Types

- **Components**: During globalization via `Globalizing<C>`
- **Resources**: During creation via `InProgressResourceBuilder<T>`

## Usage Pattern

1. Use the builder pattern to configure your component or resource
2. Call `set_init_metadata()` with a vector of metadata tuples
3. Each tuple contains: (key, value, lock_flag)
4. Continue with the normal builder chain (globalize, create, etc.)

## Locking Behavior

- `lock_flag = true`: Metadata cannot be changed after initialization
- `lock_flag = false`: Metadata can be modified later via standard Scrypto methods
*/

use scrypto::prelude::*;

/// Trait for setting metadata during component and resource initialization.
///
/// This trait provides a fluent interface for batch metadata setting that integrates
/// with Scrypto's builder patterns. It allows setting multiple metadata entries with
/// individual lock control in a single operation.
///
/// The trait is designed to be called during the initialization phase of components
/// and resources, before they are finalized (globalized or created).
pub trait MetadataSetter {
  /// Sets metadata entries during initialization with optional locking.
  ///
  /// This method allows batch setting of metadata with granular lock control.
  /// Locked metadata cannot be changed after initialization, while unlocked
  /// metadata remains modifiable through standard Scrypto metadata methods.
  ///
  /// # Arguments
  /// * `metadata` - Vector of tuples containing:
  ///   - `String`: The metadata key
  ///   - `MetadataValue`: The metadata value (any valid Scrypto metadata type)
  ///   - `bool`: Whether to lock this metadata entry (true = locked, false = unlocked)
  ///
  /// # Returns
  /// Returns `Self` to enable method chaining with the builder pattern
  ///
  fn set_init_metadata(self, metadata: Vec<(String, MetadataValue, bool)>) -> Self;
}

/// Implementation of `MetadataSetter` for component globalization.
///
/// This implementation allows setting metadata during the component globalization
/// phase, which is the final step before a component becomes available on the ledger.
/// The metadata is set as part of the component's module configuration.
impl<C: HasStub + HasMethods> MetadataSetter for Globalizing<C> {
  /// Sets metadata on a component during globalization.
  ///
  /// This method creates a metadata module configuration with the specified
  /// metadata entries and their lock states, then applies it to the globalizing
  /// component. The component can then be finalized with `.globalize()`.
  ///
  /// # Arguments
  /// * `metadata` - Vector of metadata tuples to set on the component
  ///
  /// # Returns
  /// The `Globalizing<C>` instance with metadata configured for method chaining
  ///
  fn set_init_metadata(self, metadata: Vec<(String, MetadataValue, bool)>) -> Self {
    // Create a new metadata module configuration
    let mut metadata_init = ModuleConfig {
      init: MetadataInit::new(),
      roles: RoleAssignmentInit::new(),
    };

    // Process each metadata entry according to its lock preference
    metadata.into_iter().for_each(|(key, value, lock)| {
      if lock {
        // Set and immediately lock the metadata - cannot be changed later
        metadata_init.init.set_and_lock_metadata(key, value);
      } else {
        // Set metadata but leave it unlocked - can be modified later
        metadata_init.init.set_metadata(key, value);
      }
    });

    // Apply the metadata configuration and return the updated globalizing instance
    self.metadata(metadata_init)
  }
}

/// Implementation of `MetadataSetter` for resource creation.
///
/// This implementation allows setting metadata during resource creation through
/// the resource builder pattern. The metadata is applied before the resource
/// is finalized with creation methods like `create_with_no_initial_supply()` or
/// `mint_initial_supply()`.
impl<T: AnyResourceType> MetadataSetter for InProgressResourceBuilder<T> {
  /// Sets metadata on a resource during creation.
  ///
  /// This method creates a metadata module configuration with the specified
  /// metadata entries and their lock states, then applies it to the resource
  /// builder. The resource can then be finalized with appropriate creation methods.
  ///
  /// # Arguments
  /// * `metadata` - Vector of metadata tuples to set on the resource
  ///
  /// # Returns
  /// The `InProgressResourceBuilder<T>` instance with metadata configured for method chaining
  ///
  fn set_init_metadata(self, metadata: Vec<(String, MetadataValue, bool)>) -> Self {
    // Create a new metadata module configuration
    let mut metadata_init = ModuleConfig {
      init: MetadataInit::new(),
      roles: RoleAssignmentInit::new(),
    };

    // Process each metadata entry according to its lock preference
    metadata.into_iter().for_each(|(key, value, lock)| {
      if lock {
        // Set and immediately lock the metadata - cannot be changed later
        metadata_init.init.set_and_lock_metadata(key, value);
      } else {
        // Set metadata but leave it unlocked - can be modified later
        metadata_init.init.set_metadata(key, value);
      }
    });

    // Apply the metadata configuration and return the updated resource builder
    self.metadata(metadata_init)
  }
}
