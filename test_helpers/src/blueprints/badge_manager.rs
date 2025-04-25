use common::prelude::*;
use scrypto::prelude::*;

struct CreateNftBadgeRuleInput {
  minter: Option<AccessRule>,
  minter_updater: Option<AccessRule>,
  burner: Option<AccessRule>,
  burner_updater: Option<AccessRule>,
  depositor: Option<AccessRule>,
  depositor_updater: Option<AccessRule>,
  withdrawer: Option<AccessRule>,
  withdrawer_updater: Option<AccessRule>,
  recaller: Option<AccessRule>,
  recaller_updater: Option<AccessRule>,
  freezer: Option<AccessRule>,
  freezer_updater: Option<AccessRule>,
  non_fungible_data_updater: Option<AccessRule>,
  non_fungible_data_updater_updater: Option<AccessRule>,
}

#[blueprint]
#[types(EmptyBadgeData)]
mod badge_manager {

  enable_method_auth! {
    roles {
      // Admin role:
      // - Can mint, recall and burn config moderator badges
      config_admin => updatable_by: [config_admin];

      // Fee Admin role:
      // - Can mint, recall and burn fee collector badges
      fee_manager => updatable_by: [fee_manager];
    },

    methods {
      mint_moderator_badge => restrict_to: [config_admin];
      mint_fee_collector_badge => restrict_to: [fee_manager];
    }

  }

  struct BadgeManager {
    config_moderator_res_manager: NonFungibleResourceManager,
    fee_collector_res_manager: NonFungibleResourceManager,
  }

  impl BadgeManager {
    pub fn instantiate(
      owner_badge_metadata: Vec<(String, MetadataValue, bool)>,
      admin_badge_metadata: Vec<(String, MetadataValue, bool)>,
      moderator_badge_metadata: Vec<(String, MetadataValue, bool)>,
      fee_admin_badge_metadata: Vec<(String, MetadataValue, bool)>,
      fee_collector_badge_metadata: Vec<(String, MetadataValue, bool)>,
    ) -> (
      Global<BadgeManager>,
      Option<NonFungibleBucket>,
      Option<NonFungibleBucket>,
      Option<NonFungibleBucket>,
      Option<NonFungibleBucket>,
      Option<NonFungibleBucket>,
    ) {
      // * Create owner badge * //

      // Get address reservation for the admin badge resource address
      let (owner_badge_address_reservation, super_owner_badge_res_address) = Runtime::allocate_non_fungible_address();

      // Super admin will be able to update mint new admin badges and will be owner of
      // all resources and components

      let owner_role = OwnerRole::Updatable(rule!(require(super_owner_badge_res_address)));

      let (owner_badge, _) = Self::create_nft_badge_badge(owner_role.clone(), Some(owner_badge_address_reservation), owner_badge_metadata, 1);

      // * Create admin badge * //

      // Admin will be able to create lending pools, update pool configurations and
      // update operating status

      let (config_admin_badge, config_admin_res_manager) = Self::create_nft_badge_badge(owner_role.clone(), None, admin_badge_metadata, 1);

      // * Create moderator badge * //

      // Moderator will be able to update operating status if the updates are not
      // locked by an admin

      let (config_moderator_badge, config_moderator_res_manager) = Self::create_nft_badge_badge_advanced(
        owner_role.clone(),
        None,
        moderator_badge_metadata,
        1,
        CreateNftBadgeRuleInput {
          minter: Some(rule!(require(config_admin_res_manager.address()))),
          minter_updater: Some(rule!(require(config_admin_res_manager.address()))),
          burner: Some(rule!(require(config_admin_res_manager.address()))),
          burner_updater: Some(rule!(require(config_admin_res_manager.address()))),
          depositor: Some(AccessRule::AllowAll),
          depositor_updater: Some(AccessRule::DenyAll),
          withdrawer: Some(AccessRule::AllowAll),
          withdrawer_updater: Some(AccessRule::DenyAll),
          recaller: Some(rule!(require(config_admin_res_manager.address()))),
          recaller_updater: Some(rule!(require(config_admin_res_manager.address()))),
          freezer: Some(rule!(require(config_admin_res_manager.address()))),
          freezer_updater: Some(rule!(require(config_admin_res_manager.address()))),
          non_fungible_data_updater: Some(AccessRule::DenyAll),
          non_fungible_data_updater_updater: Some(AccessRule::DenyAll),
        },
      );

      // PROTOCOL FEE MANAGEMENT //

      // * Create protocol fee admin badge * //
      let (fee_admin_badge, fee_admin_res_manager) = Self::create_nft_badge_badge(owner_role.clone(), None, fee_admin_badge_metadata, 1);

      // * Create protocol fee collector badge * //

      let (fee_collector_badge, fee_collector_res_manager) = Self::create_nft_badge_badge_advanced(
        owner_role.clone(),
        None,
        fee_collector_badge_metadata,
        1,
        CreateNftBadgeRuleInput {
          minter: Some(rule!(require(fee_admin_res_manager.address()))),
          minter_updater: Some(rule!(require(fee_admin_res_manager.address()))),
          burner: Some(rule!(require(fee_admin_res_manager.address()))),
          burner_updater: Some(rule!(require(fee_admin_res_manager.address()))),
          depositor: Some(AccessRule::AllowAll),
          depositor_updater: Some(AccessRule::DenyAll),
          withdrawer: Some(AccessRule::AllowAll),
          withdrawer_updater: Some(AccessRule::DenyAll),
          recaller: Some(rule!(require(fee_admin_res_manager.address()))),
          recaller_updater: Some(rule!(require(fee_admin_res_manager.address()))),
          freezer: Some(rule!(require(fee_admin_res_manager.address()))),
          freezer_updater: Some(rule!(require(fee_admin_res_manager.address()))),
          non_fungible_data_updater: Some(AccessRule::DenyAll),
          non_fungible_data_updater_updater: Some(AccessRule::DenyAll),
        },
      );

      let component = (Self {
        config_moderator_res_manager,
        fee_collector_res_manager,
      })
      .instantiate()
      .prepare_to_globalize(owner_role)
      // .with_address(component_address_reservation)
      .roles(roles! {
        config_admin => rule!(require(config_admin_res_manager.address()));
        fee_manager => rule!(require(fee_admin_res_manager.address()));
      })
      .globalize();

      (
        component,
        owner_badge,
        config_admin_badge,
        config_moderator_badge,
        fee_admin_badge,
        fee_collector_badge,
      )
    }

    pub fn mint_moderator_badge(&self) -> NonFungibleBucket {
      self.config_moderator_res_manager.mint_ruid_non_fungible(EmptyBadgeData {})
    }

    pub fn mint_fee_collector_badge(&self) -> NonFungibleBucket {
      self.fee_collector_res_manager.mint_ruid_non_fungible(EmptyBadgeData {})
    }

    fn create_nft_badge_badge(
      owner_role: OwnerRole,
      address_reservation: Option<GlobalAddressReservation>,
      metadata: Vec<(String, MetadataValue, bool)>,
      count: u8,
    ) -> (Option<NonFungibleBucket>, NonFungibleResourceManager) {
      let builder = ResourceBuilder::new_integer_non_fungible_with_registered_type::<EmptyBadgeData>(owner_role);

      let mut builder = if let Some(address_reservation) = address_reservation {
        builder.with_address(address_reservation)
      } else {
        builder
      };

      builder = builder.set_init_metadata(metadata);

      if count > 0 {
        let badges = builder.mint_initial_supply([(0.into(), EmptyBadgeData {})]);
        let res_manager: NonFungibleResourceManager = badges.resource_address().into();

        (Some(badges), res_manager)
      } else {
        (None, builder.create_with_no_initial_supply())
      }
    }

    fn create_nft_badge_badge_advanced(
      owner_role: OwnerRole,
      address_reservation: Option<GlobalAddressReservation>,
      metadata: Vec<(String, MetadataValue, bool)>,
      count: u8,
      rules: CreateNftBadgeRuleInput,
    ) -> (Option<NonFungibleBucket>, NonFungibleResourceManager) {
      let builder = ResourceBuilder::new_integer_non_fungible_with_registered_type::<EmptyBadgeData>(owner_role);

      let mut builder = (if let Some(address_reservation) = address_reservation {
        builder.with_address(address_reservation)
      } else {
        builder
      })
      .mint_roles(mint_roles! {
        minter => rules.minter;
        minter_updater => rules.minter_updater;
      })
      .burn_roles(burn_roles! {
        burner => rules.burner;
        burner_updater => rules.burner_updater;
      })
      .deposit_roles(deposit_roles! {
        depositor => rules.depositor;
        depositor_updater => rules.depositor_updater;
      })
      .withdraw_roles(withdraw_roles! {
        withdrawer => rules.withdrawer;
        withdrawer_updater => rules.withdrawer_updater;
      })
      .recall_roles(recall_roles! {
        recaller => rules.recaller;
        recaller_updater => rules.recaller_updater;
      })
      .freeze_roles(freeze_roles! {
        freezer => rules.freezer;
        freezer_updater => rules.freezer_updater;
      })
      .non_fungible_data_update_roles(non_fungible_data_update_roles! {
        non_fungible_data_updater => rules.non_fungible_data_updater;
        non_fungible_data_updater_updater => rules.non_fungible_data_updater_updater;
      });

      builder = builder.set_init_metadata(metadata);

      if count > 0 {
        let badges = builder.mint_initial_supply([(0.into(), EmptyBadgeData {})]);
        let res_manager: NonFungibleResourceManager = badges.resource_address().into();

        (Some(badges), res_manager)
      } else {
        (None, builder.create_with_no_initial_supply())
      }
    }
  }
}
