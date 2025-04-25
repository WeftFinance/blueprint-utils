use common::prelude::CacheEntry;
use lending_pool_proxy::LendingPoolProxy;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AuthBadgeData {}

#[derive(ScryptoSbor, Clone)]
pub struct PriceInfo {
  pub timestamp: i64,
  pub price: Decimal,
}

type ComponentAndResource = (ComponentAddress, ResourceAddress);
type ComponentAndBool = (ComponentAddress, bool);

#[blueprint]
#[types(
  AuthBadgeData,
  ComponentAddress,
  ResourceAddress,
  CacheEntry<PreciseDecimal>,
  NonFungibleVault,
  ComponentAndResource,
  ComponentAndBool,
)]
mod dummy_client_app {

  enable_method_auth! {
    roles {
      client_badge_setter => updatable_by: [];
    },
    methods {
      set_client_badge => restrict_to: [client_badge_setter];
      register_lending_pool => PUBLIC;
      deposit => PUBLIC;
      withdraw => PUBLIC;
      get_loan_unit_ratio => PUBLIC;
      protected_borrow => PUBLIC;
      protected_repay => PUBLIC;
      // get_deposit_unit_address => PUBLIC;
      // get_deposit_unit_ratio => PUBLIC;
    }
  }

  struct DummyClientApp {
    default_pool: ComponentAddress,
    lending_pool_proxy: LendingPoolProxy,
    resources: KeyValueStore<ResourceAddress, FungibleVault>,
  }
  impl DummyClientApp {
    pub fn instantiate(lending_pool: Global<AnyComponent>) {
      // let (admin_badge_address_reservation, admin_badge_address) = Runtime::allocate_non_fungible_address();

      // let admin_rule = rule!(require(admin_badge_address));

      // let admin_badge = ResourceBuilder::new_integer_non_fungible_with_registered_type::<AuthBadgeData>(OwnerRole::Fixed(admin_rule.clone()))
      //   .with_address(admin_badge_address_reservation)
      //   .metadata(metadata! {
      //     init{
      //       "name" => "Resource faucet Admin Badge", updatable;
      //     }
      //   })
      //   .mint_initial_supply([(IntegerNonFungibleLocalId::from(0), AuthBadgeData {})]);

      // let client_badge: GlobalAddress = lending_pool.get_metadata("client_badge").unwrap().unwrap();

      Self {
        default_pool: lending_pool.address(),
        lending_pool_proxy: LendingPoolProxy {
          client_badges: KeyValueStore::new_with_registered_type(),
          loan_unit_ratio_cache: KeyValueStore::new_with_registered_type(),
          registered_lending_pools: KeyValueStore::new_with_registered_type(),
        },

        resources: KeyValueStore::new(),
      }
      .instantiate()
      .prepare_to_globalize(OwnerRole::None)
      .roles(roles! {
        client_badge_setter => rule!(require(global_caller(lending_pool.address())));
      })
      .globalize();
    }

    pub fn set_client_badge(&mut self, client_badge: NonFungibleBucket) {
      // if !self.lending_pool_proxy.client_badge.is_empty() {
      //   panic!("{}", "CLIENT_BADGE_ALREADY_SET");
      // }

      // if client_badge.amount() != Decimal::ONE {
      //   panic!("{}", "ONLY_SINGLE_CLIENT_BADGE_SUPPORTED");
      // }

      // self.lending_pool_proxy.client_badge.put(client_badge);

      self.lending_pool_proxy.set_client_badge(client_badge);
    }

    pub fn register_lending_pool(&mut self, lending_pool_address: ComponentAddress, client_badge_address: ResourceAddress) {
      self
        .lending_pool_proxy
        .register_lending_pool(lending_pool_address, client_badge_address)
        .expect("ERROR_IN_REGISTER_LENDING_POOL");
    }

    pub fn deposit(&mut self, resources: FungibleBucket) {
      let res_address = resources.resource_address();

      if self.resources.get(&res_address).is_none() {
        self.resources.insert(res_address, FungibleVault::with_bucket(resources));
      } else {
        self.resources.get_mut(&res_address).unwrap().put(resources);
      }
    }

    pub fn withdraw(&mut self, resource: ResourceAddress, amount: Option<Decimal>) -> FungibleBucket {
      let mut vault = self.resources.get_mut(&resource).expect("Resource not found");

      if let Some(amount) = amount {
        vault.take_advanced(amount, WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven))
      } else {
        vault.take_all()
      }
    }

    // pub fn get_deposit_unit_address(&mut self, res_address: ResourceAddress) -> Option<ResourceAddress> {
    //   self.lending_pool_proxy.get_deposit_unit_address(res_address)
    // }

    pub fn get_loan_unit_ratio(&mut self, res_addresses: IndexSet<ResourceAddress>) -> IndexMap<ResourceAddress, PreciseDecimal> {
      self.lending_pool_proxy.get_loan_unit_ratio(self.default_pool, res_addresses)
    }

    // pub fn get_deposit_unit_ratio(&mut self, res_addresses: IndexSet<ResourceAddress>) -> IndexMap<ResourceAddress, PreciseDecimal> {
    //   self.lending_pool_proxy.get_deposit_unit_ratio(res_addresses)
    // }

    pub fn protected_borrow(&mut self, resources: IndexMap<ResourceAddress, Decimal>) {
      self
        .lending_pool_proxy
        .protected_borrow(self.default_pool, resources)
        .into_iter()
        .for_each(|(bucket, _)| self.deposit(bucket));
    }

    pub fn protected_repay(&mut self, resources: IndexMap<ResourceAddress, Decimal>) {
      let repays = resources
        .into_iter()
        .map(|(res, amount)| (self.withdraw(res, Some(amount)), None::<Decimal>))
        .collect::<Vec<_>>();

      self
        .lending_pool_proxy
        .protected_repay(self.default_pool, repays)
        .into_iter()
        .for_each(|(_, bucket, _)| self.deposit(bucket));
    }
  }
}
