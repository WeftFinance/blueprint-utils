extern crate proc_macro;
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use scrypto::prelude::*;
use syn::*;

/// Generates code for the `update` and `check` methods of a struct
/// that implements the `GenerateConfig` trait.
///
/// The generated code is based on the fields of the struct and the
/// `check` attribute.
#[proc_macro_derive(GenerateConfig, attributes(check))]
pub fn generate_config(input: TokenStream) -> TokenStream {
  match generate_config_impl(input) {
    Ok(tokens) => tokens,
    Err(error) => error.to_compile_error().into(),
  }
}

fn generate_config_impl(input: TokenStream) -> std::result::Result<TokenStream, syn::Error> {
  let input = syn::parse::<DeriveInput>(input)?;
  let name = &input.ident;

  // The fields of the struct
  let fields = match input.data {
    Data::Struct(ref data_struct) => match data_struct.fields {
      // `Fields::Named` is the type of the fields of a struct that has named fields
      Fields::Named(ref fields_named) => &fields_named.named,
      _ => return Err(syn::Error::new_spanned(&input, "GenerateConfig can only be used with named fields")),
    },
    _ => return Err(syn::Error::new_spanned(&input, "GenerateConfig can only be used with structs")),
  };

  // `update_enum_name` is the name of the enum that will be generated
  // for example, if the struct is named `Config`, `update_enum_name` will be `UpdateConfigInput`
  let update_enum_name = format_ident!("Update{}Input", name);

  // Pre-allocate vectors with capacity for better performance
  let field_count = fields.len();
  let mut update_enum_variants = Vec::with_capacity(field_count);
  let mut update_impl_match_arms = Vec::with_capacity(field_count * 2); // Maps need 2 arms
  let mut check_calls = Vec::with_capacity(field_count);

  // Iterate over the fields of the struct
  for field in fields {
    let field_name = field
      .ident
      .as_ref()
      .ok_or_else(|| syn::Error::new_spanned(field, "Field must have a name"))?;
    let field_type = &field.ty;

    // Cache string conversion for better performance
    let field_name_str = field_name.to_string();
    let variant_name = format_ident!("{}", field_name_str.to_pascal_case());

    // If the field has a `check` attribute, `check_attr` will be `Some`
    let check_attr = field.attrs.iter().find(|attr| attr.path.is_ident("check"));

    // `check_fn` is a closure that will be called in the `check` method
    let check_fn = if let Some(attr) = check_attr {
      parse_check_attribute(attr, field_type)
    } else {
      quote! { |val| true}
    };

    let (variant, match_arm) = match field_type {
      Type::Path(type_path) => {
        let last_segment = type_path.path.segments.last().unwrap();
        let type_name = last_segment.ident.to_string();

        if matches!(type_name.as_str(), "BTreeSet" | "HashSet" | "IndexSet") {
          let args = extract_generic_args(&last_segment.arguments)?;
          if args.is_empty() {
            return Err(syn::Error::new_spanned(last_segment, "Set types require a generic parameter"));
          }
          generate_set_field_code(args[0], &variant_name, field_name, &update_enum_name)
        } else if matches!(type_name.as_str(), "BTreeMap" | "HashMap" | "IndexMap") {
          let args = extract_generic_args(&last_segment.arguments)?;
          if args.len() < 2 {
            return Err(syn::Error::new_spanned(last_segment, "Map types require two generic parameters"));
          }
          generate_map_field_code(args[0], args[1], &variant_name, field_name, &update_enum_name)
        } else {
          generate_simple_field_code(field_type, &variant_name, field_name, &update_enum_name)
        }
      }
      _ => generate_simple_field_code(field_type, &variant_name, field_name, &update_enum_name),
    };

    update_enum_variants.push(variant);
    update_impl_match_arms.push(match_arm);

    // Add the check call to the `check` method
    check_calls.push(quote! {
        if !(#check_fn)(&self.#field_name) {
            return Err(
                format!(
                    "Invalid {}::{}",
                    std::stringify!(#name),
                    std::stringify!(#field_name)
                )
            );
        }
    });
  }

  // Generate the expanded code
  let expanded = quote! {
      #[derive(ScryptoSbor, ManifestSbor, Debug, Clone, PartialEq, Eq, Hash)]
      pub enum #update_enum_name {
          #(#update_enum_variants),*
      }

      impl #name {
          /// Applies updates in order, then validates via `check()`.
          ///
          /// Contract: mutate-then-validate. If validation fails, changes remain
          /// applied to `self` in-memory; callers should only persist state after a
          /// successful `Ok(())` or use a wrapper that re-validates before persistence.
          #[inline]
          pub fn update(&mut self, config_inputs: IndexSet<#update_enum_name>) -> Result<(), String> {
              for config_input in config_inputs {
                  match config_input {
                      #(#update_impl_match_arms),*
                  };
              }

              self.check()?;

              Ok(())
          }

          /// Validates the current configuration state. Returns `Err(String)` with a
          /// concise message when any field-level predicate fails.
          #[inline]
          pub fn check(&self) -> Result<(),String> {
              #(#check_calls)*

              Ok(())
          }
      }
  };

  Ok(TokenStream::from(expanded))
}

fn generate_set_field_code(
  inner_type: &syn::GenericArgument,
  variant_name: &Ident,
  field_name: &Ident,
  update_enum_name: &Ident,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
  // Keep 1:1 variant-to-field mapping: payload is (T, bool)
  // bool = true => Add; false => Remove
  let variant = quote! {
    #variant_name((#inner_type, bool))
  };

  let match_arm = quote! {
    #update_enum_name::#variant_name((value, is_add)) => {
      if is_add { self.#field_name.insert(value); } else { self.#field_name.remove(&value); }
    }
  };

  (variant, match_arm)
}

fn generate_map_field_code(
  key_type: &syn::GenericArgument,
  value_type: &syn::GenericArgument,
  variant_name: &Ident,
  field_name: &Ident,
  update_enum_name: &Ident,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
  let variant = quote! {
    #variant_name(#key_type, Option<#value_type>)
  };

  let match_arm = quote! {
    #update_enum_name::#variant_name(key, Some(value)) => {
      self.#field_name.insert(key, value);
    },
    #update_enum_name::#variant_name(key, None) => {
      self.#field_name.remove(&key);
    }
  };

  (variant, match_arm)
}

fn generate_simple_field_code(
  field_type: &Type,
  variant_name: &Ident,
  field_name: &Ident,
  update_enum_name: &Ident,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
  let variant = quote! {
    #variant_name(#field_type)
  };

  let match_arm = quote! {
    #update_enum_name::#variant_name(value) => {
      self.#field_name = value;
    }
  };

  (variant, match_arm)
}

fn extract_generic_args(args: &syn::PathArguments) -> std::result::Result<Vec<&syn::GenericArgument>, syn::Error> {
  if let syn::PathArguments::AngleBracketed(args) = args {
    Ok(args.args.iter().collect())
  } else {
    Err(syn::Error::new_spanned(args, "Expected generic arguments"))
  }
}

fn parse_check_attribute(attr: &Attribute, ty: &Type) -> proc_macro2::TokenStream {
  let meta = attr.parse_meta().expect("Failed to parse attribute");
  if let Meta::NameValue(name_value) = meta {
    if let Lit::Str(lit_str) = name_value.lit {
      let check_fn: proc_macro2::TokenStream = lit_str.value().parse().expect("Failed to parse check function");
      return quote! { |val:&#ty| #check_fn };
    }
  }
  panic!("Invalid check attribute syntax");
}
