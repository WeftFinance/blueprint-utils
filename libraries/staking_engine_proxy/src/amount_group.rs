use indexmap::map::Entry;
use scrypto::prelude::*;

/// A group of amounts associated with `(u8, ResourceAddress)` keys.
#[derive(ScryptoSbor, Clone, Debug, Default)]
pub struct AmountGroup<T>
where
  T: Default + Clone + ScryptoSbor,
{
  pub values: IndexMap<ResourceAddress, T>,
}

impl<T> AmountGroup<T>
where
  T: Default + Clone + ScryptoSbor,
{
  /// Creates a new empty `AmountGroup`.
  pub fn new() -> Self {
    Self { values: IndexMap::new() }
  }

  /// Retrieves a mutable reference to the value corresponding to the given key,
  /// inserting the default value if the key does not exist.
  pub fn get_mut_item(&mut self, key: &ResourceAddress) -> &mut T {
    self.values.entry(*key).or_default()
  }

  /// Retrieves a clone of the value corresponding to the given key,
  /// returning the default value if the key does not exist.
  pub fn get_item(&self, key: &ResourceAddress) -> T {
    self.values.get(key).cloned().unwrap_or_default()
  }

  /// Returns the number of unique resources in the group.
  pub fn resource_count(&self) -> usize {
    self.values.len()
  }

  pub fn select(&self, key: &IndexSet<ResourceAddress>) -> AmountGroup<T> {
    let mut result = AmountGroup::new();
    for k in key.iter() {
      result.values.insert(*k, self.get_item(k));
    }
    result
  }
}

impl AmountGroup<Decimal> {
  /// Subtracts all values in `other` from the current group.
  pub fn sub_all(&mut self, other: &AmountGroup<Decimal>) {
    self.apply_all(other, |a, b| a.checked_sub(*b).unwrap());
  }

  /// Adds all values in `other` to the current group.
  pub fn add_all(&mut self, other: &AmountGroup<Decimal>) {
    self.apply_all(other, |a, b| a.checked_add(*b).unwrap());
  }
}

impl AmountGroup<PreciseDecimal> {
  /// Adds all values in `other` to the current group, converting them to `PreciseDecimal` if needed.
  pub fn add_all(&mut self, other: &AmountGroup<Decimal>) {
    self.apply_all(other, |a, b| a.checked_add(PreciseDecimal::from(*b)).unwrap());
  }

  /// Subtracts all values in `other` from the current group, converting them to `PreciseDecimal` if needed.
  pub fn sub_all(&mut self, other: &AmountGroup<Decimal>) {
    self.apply_all(other, |a, b| a.checked_sub(PreciseDecimal::from(*b)).unwrap());
  }
}

impl<T> AmountGroup<T>
where
  T: Default + Clone + ScryptoSbor,
{
  /// Applies a binary operation on all values in `other`, modifying the current group in place.
  /// Handles cases where `T` and `U` are different types (e.g., `Decimal` and `PreciseDecimal`).
  fn apply_all<U, F>(&mut self, other: &AmountGroup<U>, op: F)
  where
    U: Clone + ScryptoSbor + Default,
    F: Fn(&T, &U) -> T,
  {
    for (key, amount) in &other.values {
      match self.values.entry(*key) {
        Entry::Occupied(mut entry) => {
          *entry.get_mut() = op(entry.get(), amount);
        }
        Entry::Vacant(entry) => {
          entry.insert(op(&T::default(), amount));
        }
      }
    }
  }
}
