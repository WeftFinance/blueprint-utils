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
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;

  // The fields of the struct
  let fields = match input.data {
    Data::Struct(ref data_struct) => match data_struct.fields {
      // `Fields::Named` is the type of the fields of a struct that has named fields
      Fields::Named(ref fields_named) => &fields_named.named,
      _ => panic!("GenerateConfig can only be used with named fields"),
    },
    _ => panic!("GenerateConfig can only be used with structs"),
  };

  // `update_enum_name` is the name of the enum that will be generated
  // for example, if the struct is named `Config`, `update_enum_name` will be `UpdateConfigInput`
  let update_enum_name = format_ident!("Update{}Input", name);

  // `update_enum_variants` is a vec of the variants of the enum
  // for example, if the struct has a field named `foo`, `update_enum_variants` will contain the variant `Foo`
  let mut update_enum_variants = Vec::new();

  // `update_impl_match_arms` is a vec of the match arms for the `update` method
  // for example, if the struct has a field named `foo`, `update_impl_match_arms` will contain the match arm `Foo => self.foo = value;`
  let mut update_impl_match_arms = Vec::new();

  // `check_calls` is a vec of the checks that will be done in the `check` method
  // for example, if the struct has a field named `foo`, `check_calls` will contain the check `if !(self.check_foo)(&self.foo) { ... }`
  let mut check_calls = Vec::new();

  // Iterate over the fields of the struct
  for field in fields {
    let field_name = &field.ident;
    let field_type = &field.ty;

    // `variant_name` is the name of the variant of the enum
    // for example, if the field is named `foo`, `variant_name` will be `Foo`
    let variant_name = format_ident!("{}", field_name.as_ref().unwrap().to_string().to_pascal_case());

    // If the field has a `check` attribute, `check_attr` will be `Some`
    let check_attr = field.attrs.iter().find(|attr| attr.path.is_ident("check"));

    // `check_fn` is a closure that will be called in the `check` method
    // if the field has a `check` attribute, `check_fn` will be the closure specified in the attribute
    // otherwise, `check_fn` will be `|val| true`
    let check_fn = if let Some(attr) = check_attr {
      parse_check_attribute(attr, field_type)
    } else {
      quote! { |val| true}
    };

    match field_type {
      Type::Path(type_path) => {
        let last_segment = type_path.path.segments.last().unwrap();
        match last_segment.ident.to_string().as_str() {
          "BTreeSet" | "HashSet" | "IndexSet" => {
            // `inner_type` is the type of the elements of the set
            // for example, if the field is a `BTreeSet<i32>`, `inner_type` will be `i32`
            let inner_type = if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
              &args.args[0]
            } else {
              panic!("Expected Set to have a generic parameter");
            };

            // Add the variant to the enum
            update_enum_variants.push(quote! {
                #variant_name(UpdateSetInput<#inner_type>)
            });

            // Add the match arm to the `update` method
            update_impl_match_arms.push(quote! {
                #update_enum_name::#variant_name(UpdateSetInput::Add(value)) => {
                    self.#field_name.insert(value);
                },
                #update_enum_name::#variant_name(UpdateSetInput::Remove(value)) => {
                    self.#field_name.remove(&value);
                }
            });
          }
          "BTreeMap" | "HashMap" | "IndexMap" => {
            // `inner_types` is a tuple of the types of the elements of the map
            // for example, if the field is a `BTreeMap<String,i32>`, `inner_types` will be `(String, i32)`
            let inner_types = if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
              let mut types = args.args.iter();
              (types.next().unwrap(), types.next().unwrap())
            } else {
              panic!("Expected IndexMap to have two generic parameters");
            };

            let (key_type, value_type) = inner_types;

            // Add the variant to the enum
            update_enum_variants.push(quote! {
                #variant_name(#key_type, Option<#value_type>)
            });

            // Add the match arm to the `update` method
            update_impl_match_arms.push(quote! {
                #update_enum_name::#variant_name(key, Some(value)) => {
                    self.#field_name.insert(key, value);
                },
                #update_enum_name::#variant_name(key, None) => {
                    self.#field_name.remove(&key);
                }
            });
          }
          _ => {
            // Add the variant to the enum
            update_enum_variants.push(quote! {
                #variant_name(#field_type)
            });

            // Add the match arm to the `update` method
            update_impl_match_arms.push(quote! {
                #update_enum_name::#variant_name(value) => {
                    self.#field_name = value;
                }
            });
          }
        }
      }
      _ => {
        // Add the variant to the enum
        update_enum_variants.push(quote! {
            #variant_name(#field_type)
        });

        // Add the match arm to the `update` method
        update_impl_match_arms.push(quote! {
            #update_enum_name::#variant_name(value) => {
                self.#field_name = value;
            }
        });
      }
    }

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
          pub fn update(&mut self, config_inputs: IndexSet<#update_enum_name>) -> Result<(), String> {
              for config_input in config_inputs {
                  match config_input {
                      #(#update_impl_match_arms),*
                  };
              }

              self.check()?;

              Ok(())
          }

          pub fn check(&self) -> Result<(),String> {
              #(#check_calls)*

              Ok(())
          }
      }
  };

  TokenStream::from(expanded)
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
