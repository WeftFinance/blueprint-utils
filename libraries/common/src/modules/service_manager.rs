/*!
# Service Manager Module

This module provides a comprehensive service management system for Scrypto blueprints,
enabling fine-grained control over service availability and access permissions.

## Key Features

- **Multi-service Management**: Track status of multiple services per entity
- **Role-based Access Control**: Three-tier system (Admin/Moderator/User)
- **Service Locking**: Admin can lock services to prevent moderator changes
- **Automatic Service Discovery**: Macro-generated service enums with variant listing
- **Entity-keyed Management**: Manage services across different entities/pools/components

## Usage Pattern

1. Define services using `generate_service_variants!` macro
2. Create `ServiceManager` instance for your blueprint
3. Initialize services for each entity with `set_entry()`
4. Control service availability with role-based `update()` calls
5. Check service status before operations with `check()` or `assert()`

## Access Control

- **Admin**: Can enable/disable any service and lock/unlock them
- **Moderator**: Can enable/disable unlocked services only
- **User**: Cannot change service status (read-only access)
*/

use anyhow::{anyhow, ensure, Result};
use indexmap::Equivalent;
use scrypto::prelude::rust::hash::Hash;
use scrypto::prelude::*;

/// Represents the operating status of a single service.
///
/// This struct tracks both whether a service is enabled and whether it's locked
/// by an administrator. Locked services can only be modified by admin roles.
#[derive(ScryptoSbor, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperatingStatus {
  /// Whether the service is currently enabled and available for use
  enabled: bool,
  /// Whether the service is locked by admin, preventing moderator modifications
  locked: bool,
}

/// Defines the source and authority level for service status changes.
///
/// This enum implements a three-tier access control system where admins have
/// full control, moderators have limited control, and the locking mechanism
/// provides additional security for critical services.
#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub enum StatusChangeType {
  /// Admin sets the status and locks it - only admin can change it in the future
  AdminSetAndLock,
  /// Admin sets the status but leaves it unlocked - admin and moderator can change it
  AdminSetAndUnlock,
  /// Moderator sets the status - only works if service is not locked by admin
  ModeratorSet,
}

/// Trait that service enums must implement to be managed by the service system.
///
/// This trait provides a way to discover all available service variants at runtime,
/// which is essential for initializing service status maps and validation.
///
/// Typically implemented automatically by the `generate_service_variants!` macro.
pub trait ServiceVariantProvider {
  /// Returns a vector containing all possible variants of this service type.
  ///
  /// This method is used to initialize service status maps with all services
  /// set to their default state (enabled and unlocked).
  fn variants() -> Vec<Self>
  where
    Self: Sized;
}

/// Container for managing the operating status of multiple services.
///
/// This struct maintains a mapping from service types to their current operating
/// status, providing methods to check availability and modify status with proper
/// access control enforcement.
#[derive(ScryptoSbor, Default, Debug, Clone)]
pub struct ServiceStatus<T: ScryptoSbor + ServiceVariantProvider + Eq + Clone + Hash>(pub IndexMap<T, OperatingStatus>);

impl<T> ServiceStatus<T>
where
  T: ScryptoSbor + ServiceVariantProvider + Eq + Debug + Clone + Hash + Equivalent<T>,
{
  /// Creates a new ServiceStatus with all services enabled and unlocked by default.
  ///
  /// This method uses the `ServiceVariantProvider::variants()` method to discover
  /// all available services and initializes them in an enabled, unlocked state.
  pub fn new() -> Self {
    let mut services = IndexMap::new();
    let default_value = OperatingStatus {
      enabled: true,
      locked: false,
    };

    for service in T::variants() {
      services.insert(service, default_value);
    }

    Self(services)
  }

  /// Updates the status of a specific service with role-based access control.
  ///
  /// This method enforces the three-tier access control system:
  /// - Admin operations can always succeed and can lock/unlock services
  /// - Moderator operations only succeed if the service is not locked by admin
  ///
  /// # Arguments
  /// * `service` - The service to modify
  /// * `new_status` - Whether to enable (true) or disable (false) the service
  /// * `status_change_type` - The authority level and locking behavior for this change
  ///
  /// # Returns
  /// * `Ok(())` if the status change was successful
  /// * `Err` if a moderator tries to modify a locked service
  pub fn set_status(&mut self, service: T, new_status: bool, status_change_type: StatusChangeType) -> Result<()> {
    match status_change_type {
      StatusChangeType::AdminSetAndLock => self.admin_set_status(service, new_status, true),
      StatusChangeType::AdminSetAndUnlock => self.admin_set_status(service, new_status, false),
      StatusChangeType::ModeratorSet => self.moderator_set_status(service, new_status),
    }
  }

  /// Checks if a service is currently enabled and available for use.
  ///
  /// This is a non-panicking check that returns a boolean indicating whether
  /// the service is enabled. Use this when you want to conditionally execute
  /// service-dependent code.
  ///
  /// # Arguments
  /// * `service` - The service to check
  ///
  /// # Returns
  /// * `true` if the service is enabled and available
  /// * `false` if the service is disabled or not found
  pub fn check(&self, service: &T) -> bool {
    self.0.get(service).is_some_and(|status| status.enabled)
  }

  /// Asserts that a service is active, returning an error if it's not.
  ///
  /// This method provides a fail-fast approach for service checking. Use this
  /// when you want to immediately abort an operation if a required service
  /// is not available.
  ///
  /// # Arguments
  /// * `service` - The service to assert is active
  ///
  /// # Returns
  /// * `Ok(())` if the service is enabled and available
  /// * `Err` with a descriptive message if the service is disabled or not found
  pub fn assert_active(&self, service: &T) -> Result<()> {
    ensure!(self.check(service), "The service is not active: ({:?})", service);

    Ok(())
  }

  /// Internal method for admin-level status changes.
  ///
  /// Admins have unrestricted access and can modify any service regardless of
  /// its current lock status. They can also change the lock status itself.
  ///
  /// # Arguments
  /// * `service` - The service to modify
  /// * `new_status` - Whether to enable or disable the service
  /// * `admin_lock` - Whether to lock the service after this change
  fn admin_set_status(&mut self, service: T, new_status: bool, admin_lock: bool) -> Result<()> {
    self.0.insert(
      service,
      OperatingStatus {
        enabled: new_status,
        locked: admin_lock,
      },
    );

    Ok(())
  }

  /// Internal method for moderator-level status changes.
  ///
  /// Moderators can only modify services that are not locked by an admin.
  /// They cannot change the lock status of services and will receive an error
  /// if they attempt to modify a locked service.
  ///
  /// # Arguments
  /// * `service` - The service to modify
  /// * `new_status` - Whether to enable or disable the service
  ///
  /// # Returns
  /// * `Ok(())` if the service was successfully modified
  /// * `Err` if the service is locked by an admin
  fn moderator_set_status(&mut self, service: T, new_status: bool) -> Result<()> {
    if let Some(status) = self.0.get_mut(&service) {
      ensure!(!status.locked, "Service status locked by cannot be changed by a moderator");

      status.enabled = new_status;
    } else {
      self.0.insert(
        service,
        OperatingStatus {
          enabled: new_status,
          locked: false,
        },
      );
    }

    Ok(())
  }
}

/// High-level service manager that manages service status across multiple entities.
///
/// This struct provides entity-keyed service management, allowing different entities
/// (such as pools, vaults, or user accounts) to have independent service configurations.
/// Each entity can have its own set of service statuses managed separately.
///
/// # Type Parameters
/// * `K` - The key type for identifying entities (e.g., pool addresses, user IDs)
/// * `T` - The service type that implements `ServiceVariantProvider`
#[derive(ScryptoSbor)]
pub struct ServiceManager<K: ScryptoSbor + Hash + Copy, T: ScryptoSbor + ServiceVariantProvider + Eq + Clone + Hash + Debug> {
  /// Key-value store mapping entity keys to their service status configurations
  entries: KeyValueStore<K, ServiceStatus<T>>,
}

impl<K: ScryptoSbor + Hash + Copy, T: ScryptoSbor + ServiceVariantProvider + Eq + Clone + Hash + Debug> ServiceManager<K, T> {
  /// Creates a new ServiceManager with the provided key-value store.
  ///
  /// # Arguments
  /// * `entries` - A KeyValueStore to persist service status data
  pub fn new(entries: KeyValueStore<K, ServiceStatus<T>>) -> Self {
    Self { entries }
  }

  /// Initializes service status for a new entity.
  ///
  /// This method creates a new ServiceStatus instance for the given key,
  /// with all services enabled and unlocked by default. Call this method
  /// when setting up services for a new entity.
  ///
  /// # Arguments
  /// * `key` - The entity identifier to initialize services for
  pub fn set_entry(&mut self, key: K) -> Result<()> {
    self.entries.insert(key, ServiceStatus::new());
    Ok(())
  }

  /// Checks if a specific service is enabled for a given entity.
  ///
  /// # Arguments
  /// * `key` - The entity identifier
  /// * `service` - The service to check
  ///
  /// # Returns
  /// * `Ok(true)` if the service is enabled
  /// * `Ok(false)` if the service is disabled
  /// * `Err` if the entity is not found
  pub fn check(&self, key: K, service: T) -> Result<bool> {
    let entry = self.entries.get(&key).ok_or_else(|| anyhow!("Service is not set"))?;
    Ok(entry.check(&service))
  }

  /// Asserts that a specific service is active for a given entity.
  ///
  /// This method provides fail-fast behavior for service validation.
  ///
  /// # Arguments
  /// * `key` - The entity identifier
  /// * `service` - The service to assert is active
  ///
  /// # Returns
  /// * `Ok(())` if the service is enabled
  /// * `Err` if the service is disabled or the entity is not found
  pub fn assert(&self, key: K, service: &T) -> Result<()> {
    let entry = self.entries.get(&key).ok_or_else(|| anyhow!("Service is not set"))?;
    entry.assert_active(service)
  }

  /// Updates the status of a specific service for a given entity.
  ///
  /// This method applies role-based access control rules to determine if the
  /// status change is allowed.
  ///
  /// # Arguments
  /// * `key` - The entity identifier
  /// * `service` - The service to modify
  /// * `new_status` - Whether to enable (true) or disable (false) the service
  /// * `status_change_type` - The authority level and locking behavior
  ///
  /// # Returns
  /// * `Ok(())` if the status was successfully updated
  /// * `Err` if the entity is not found or access control rules prevent the change
  pub fn update(&mut self, key: K, service: T, new_status: bool, status_change_type: StatusChangeType) -> Result<()> {
    let mut entry = self.entries.get_mut(&key).ok_or_else(|| anyhow!("Service is not set"))?;
    entry.set_status(service, new_status, status_change_type)
  }
}

/// Macro to automatically generate service enums with required trait implementations.
///
/// This macro creates a service enum and automatically implements the necessary traits
/// for integration with the service management system. It generates:
///
/// 1. The enum with specified variants and derives
/// 2. A `variants()` method that returns all enum variants
/// 3. Implementation of `ServiceVariantProvider` trait
///
/// # Syntax
/// generate_service_variants!(
///     pub enum ServiceName,
///     (Derive1, Derive2, ...),
///     Variant1,
///     Variant2,
///     // ... more variants
/// );
///
/// # Example
/// generate_service_variants!(
///     pub enum LendingService,
///     (ScryptoSbor, Debug, Clone, Copy, PartialEq, Eq, Hash),
///     Supply,
///     Withdraw,
///     Borrow,
///     Repay
/// );
#[macro_export]
macro_rules! generate_service_variants {
  (
      $(#[$meta:meta])*
      $vis:vis enum $EnumName:ident,
      ($($derive:ident),*),
      $($Variant:ident),+
  ) => {
      // Generate the enum with specified metadata, visibility, and derives
      $(#[$meta])*
      #[derive($($derive),*)]
      $vis enum $EnumName {
          $($Variant),+
      }

      impl $EnumName {
          /// Returns a vector containing all variants of this service enum.
          ///
          /// This method is used by the service management system to initialize
          /// service status maps and for validation purposes.
          pub fn variants() -> Vec<$EnumName> {
              vec![
                  $($EnumName::$Variant),+
              ]
          }
      }

      // Implement the ServiceVariantProvider trait required by the service system
      impl common::modules::service_manager::ServiceVariantProvider for $EnumName {
        /// Implementation of ServiceVariantProvider trait.
        ///
        /// This delegates to the generated `variants()` method to provide
        /// the service management system with access to all service variants.
        fn variants() -> Vec<Self> {
          $EnumName::variants()
        }
      }
  };
}
