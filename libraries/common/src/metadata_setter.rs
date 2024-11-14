use scrypto::prelude::*;

pub trait MetadataSetter {
  fn set_init_metadata(self, metadata: Vec<(String, MetadataValue, bool)>) -> Self;
}

impl<C: HasStub + HasMethods> MetadataSetter for Globalizing<C> {
  fn set_init_metadata(self, metadata: Vec<(String, MetadataValue, bool)>) -> Self {
    let mut metadata_init = ModuleConfig {
      init: MetadataInit::new(),
      roles: RoleAssignmentInit::new(),
    };

    metadata.into_iter().for_each(|(key, value, lock)| {
      if lock {
        metadata_init.init.set_and_lock_metadata(key, value);
      } else {
        metadata_init.init.set_metadata(key, value);
      }
    });

    self.metadata(metadata_init)
  }
}

impl<T: AnyResourceType> MetadataSetter for InProgressResourceBuilder<T> {
  fn set_init_metadata(self, metadata: Vec<(String, MetadataValue, bool)>) -> Self {
    let mut metadata_init = ModuleConfig {
      init: MetadataInit::new(),
      roles: RoleAssignmentInit::new(),
    };

    metadata.into_iter().for_each(|(key, value, lock)| {
      if lock {
        metadata_init.init.set_and_lock_metadata(key, value);
      } else {
        metadata_init.init.set_metadata(key, value);
      }
    });

    self.metadata(metadata_init)
  }
}
