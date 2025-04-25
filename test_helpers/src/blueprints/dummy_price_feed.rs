use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AuthBadgeData {}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct UpdaterBadgeData {
  pub active: bool,
}

#[derive(ScryptoSbor, Clone)]
pub struct PriceInfo {
  pub timestamp: Instant,
  pub price: Decimal,
}

#[blueprint]
#[types(ResourceAddress, PriceInfo, UpdaterBadgeData, AuthBadgeData)]
mod dummy_price_feed {

  enable_method_auth! {
    roles {

      admin => updatable_by: [];

    },
    methods {

      update_price => restrict_to: [admin];

      get_price => PUBLIC;
      get_relative_price => PUBLIC;
      get_prices => PUBLIC;
      get_relative_prices => PUBLIC;

    }
  }

  pub struct DummyPriceFeed {
    prices: KeyValueStore<ResourceAddress, PriceInfo>,
    updater_badge_manager: NonFungibleResourceManager,
    updater_counter: u64,
  }

  impl DummyPriceFeed {
    pub fn instantiate() -> (Global<DummyPriceFeed>, NonFungibleBucket) {
      let (component_address_reservation, component_address) = Runtime::allocate_component_address(DummyPriceFeed::blueprint_id());

      let component_rule = rule!(require(global_caller(component_address)));

      let (admin_badge_address_reservation, admin_badge_address) = Runtime::allocate_non_fungible_address();

      let admin_rule = rule!(require(admin_badge_address));

      let admin_badge = ResourceBuilder::new_integer_non_fungible_with_registered_type::<AuthBadgeData>(OwnerRole::Updatable(admin_rule.clone()))
        .with_address(admin_badge_address_reservation)
        .mint_initial_supply([(0.into(), AuthBadgeData {})]);

      let updater_badge_manager =
        ResourceBuilder::new_integer_non_fungible_with_registered_type::<UpdaterBadgeData>(OwnerRole::Updatable(admin_rule.clone()))
          .mint_roles(mint_roles! {
            minter => component_rule.clone();
            minter_updater =>  rule!(deny_all);
          })
          .non_fungible_data_update_roles(non_fungible_data_update_roles! {
            non_fungible_data_updater => component_rule;
            non_fungible_data_updater_updater => rule!(deny_all);
          })
          .create_with_no_initial_supply();

      let component = Self {
        prices: KeyValueStore::new_with_registered_type(),
        updater_badge_manager,
        updater_counter: 0,
      }
      .instantiate()
      .prepare_to_globalize(OwnerRole::Fixed(admin_rule.clone()))
      .with_address(component_address_reservation)
      .enable_component_royalties(component_royalties! {
        init {
          update_price => Free, updatable;
          get_price => Free, updatable;
          get_relative_price => Free, updatable;
          get_prices => Free, updatable;
          get_relative_prices => Free, updatable;
        }
      })
      .roles(roles! {
        admin => admin_rule;
      })
      .globalize();

      (component, admin_badge)
    }

    // * Updater Methods * //

    pub fn update_price(&mut self, prices: IndexMap<ResourceAddress, Decimal>) {
      let now = Clock::current_time(TimePrecision::Minute);

      prices.iter().for_each(|(resource, price)| {
        self.prices.insert(
          *resource,
          PriceInfo {
            timestamp: now,
            price: *price,
          },
        );
      })
    }

    // * User Methods * //

    pub fn get_price(&self, quote: ResourceAddress) -> Option<PriceInfo> {
      self.prices.get(&quote).map(|p| p.clone())
    }

    pub fn get_relative_price(&self, quote: ResourceAddress, base: ResourceAddress) -> Option<PriceInfo> {
      let quote_price = self.prices.get(&quote).map(|p| p.clone());
      let base_price = self.prices.get(&base).map(|p| p.clone());
      match (base_price, quote_price) {
        (Some(base_price), Some(quote_price)) => Some(PriceInfo {
          price: quote_price.price.checked_div(base_price.price).unwrap(),
          timestamp: quote_price.timestamp.min(base_price.timestamp),
        }),
        _ => None,
      }
    }

    pub fn get_prices(&self, quotes: IndexSet<ResourceAddress>) -> IndexMap<ResourceAddress, PriceInfo> {
      quotes
        .into_iter()
        .map(|quote| (quote, self.prices.get(&quote).cloned().unwrap()))
        .collect()
    }

    pub fn get_relative_prices(&self, quotes: IndexSet<ResourceAddress>, base: ResourceAddress) -> IndexMap<ResourceAddress, Option<PriceInfo>> {
      quotes.into_iter().map(|quote| (quote, self.get_relative_price(quote, base))).collect()
    }
  }
}
