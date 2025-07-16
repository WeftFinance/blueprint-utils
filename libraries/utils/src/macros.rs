#[macro_export]
macro_rules! check_field_invalidity {
  ($config:ident, $get_default_config:ident, $field:ident, $valid_values:expr, $invalid_values:expr) => {
    paste::paste! {

        #[test]
        fn [<valid_ $field>]() {
          let default_config = $get_default_config();

          for valid_value in $valid_values {
              let config = $config {
                  $field: valid_value,
                  ..default_config.clone()
              };
             config.check().unwrap();
          }
        }

        #[test]
        fn [<invalid_ $field>]() {
          let default_config = $get_default_config();

          for invalid_value in $invalid_values {
              let config = $config {
                  $field: invalid_value,
                  ..default_config.clone()
              };
              assert!(config.check().is_err());
          }
        }
    }
  };
}

#[macro_export]
macro_rules! define_error {
    ($prefix:literal,$($name:ident,)*) => {
        $(
            pub const $name: &'static str = concat!($prefix," ", stringify!($name));
        )*
    };
}

#[macro_export]
macro_rules! create_event {
    ($name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        #[derive(ScryptoSbor, ScryptoEvent)]
        pub struct $name {
            $(pub $field: $type),*
        }
        impl  $name {
         pub fn emit(self) {
            Runtime::emit_event(self)
          }
        }
    };
    ($name:ident($data_type:ty)) => {
        #[derive(ScryptoSbor, ScryptoEvent)]
        pub struct $name (pub $data_type);
        impl $name {
         pub fn emit(self) {
            Runtime::emit_event(self)
          }
        }
    };
}

#[macro_export]
macro_rules! define_inner_error {
    ($($name:ident,)*) => {
        $(
            pub const $name: &'static str =  stringify!($name);
        )*
    };
}
