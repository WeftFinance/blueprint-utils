use scrypto::prelude::*;

/// Define an update set
#[derive(ScryptoSbor, ManifestSbor, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdateSetInput<T> {
  Remove(T),
  Add(T),
}
