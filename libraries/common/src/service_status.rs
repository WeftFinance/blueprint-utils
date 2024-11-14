use anyhow::{ensure, Result};
use indexmap::Equivalent;
use scrypto::prelude::rust::hash::Hash;
use scrypto::prelude::*;

/// Define the status of the service
#[derive(ScryptoSbor, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperatingStatus {
  /// Whether the service is enabled or not
  enabled: bool,
  /// Whether the service is locked by admin or not
  locked: bool,
}

/// Help in defining the status change source and type
/// - AdminLock: The change will be lock by admin so only admin can change the status in the future
/// - AdminUnlock: The change will be unlock so admin and moderator can change the status in the future
/// - Moderator: The change will be made by moderator an admin can override it at any time
#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub enum StatusChangeType {
  AdminSetAndLock,
  AdminSetAndUnlock,
  ModeratorSet,
}

/// Define a trait that requires a 'variants' method.
/// The 'variants' method returns the list of valid variants
/// of a service
pub trait ServiceVariantProvider {
  fn variants() -> Vec<Self>
  where
    Self: Sized;
}

/// A map of service status. The key is the service type and the value is the operating status.
#[derive(ScryptoSbor, Default, Debug, Clone)]
pub struct ServiceStatus<T: ScryptoSbor + ServiceVariantProvider + Eq + Clone + Hash>(pub IndexMap<T, OperatingStatus>);

impl<T> ServiceStatus<T>
where
  T: ScryptoSbor + ServiceVariantProvider + Eq + Debug + Clone + Hash + Equivalent<T>,
{
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

  /// Set the service status based on the status change type
  pub fn set_status(&mut self, service: T, new_status: bool, status_change_type: StatusChangeType) -> Result<()> {
    match status_change_type {
      StatusChangeType::AdminSetAndLock => self.admin_set_status(service, new_status, true),
      StatusChangeType::AdminSetAndUnlock => self.admin_set_status(service, new_status, false),
      StatusChangeType::ModeratorSet => self.moderator_set_status(service, new_status),
    }
  }

  /// check current service status and panic if the service is not active
  pub fn check(&self, service: &T) -> bool {
    self.0.get(service).map_or(false, |status| status.enabled)
  }

  /// check current service status and panic if the service is not active
  pub fn assert_active(&self, service: &T) -> Result<()> {
    ensure!(self.check(service), "The service is not active: ({:?})", service);

    Ok(())
  }

  /// set status and change lock status (admin only)
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

  /// set status (moderator only)
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

/// Macro to generate service variants
///
/// `
/// generate_service_variants!(
///   Lending, // A service
///   (Debug, Clone, PartialEq, Eq),
///   Supply, // variant 1
///   Withdraw // variant 2
/// );
/// `
#[macro_export]
macro_rules! generate_service_variants {
  (
      $(#[$meta:meta])*
      $vis:vis enum $EnumName:ident,
      ($($derive:ident),*),
      $($Variant:ident),+
  ) => {
      $(#[$meta])*
      #[derive($($derive),*)]
      $vis enum $EnumName {
          $($Variant),+
      }

      impl $EnumName {
          pub fn variants() -> Vec<$EnumName> {
              vec![
                  $($EnumName::$Variant),+
              ]
          }
      }

      impl common::service_status::ServiceVariantProvider for $EnumName {
        // The `variants` function returns the list of valid variants
        fn variants() -> Vec<Self> {
          $EnumName::variants()
        }
      }
  };
}
