use crate::utils::{AttrParams, DeriveType, State};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, Result};

pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::with_attr_params(
        input,
        trait_name,
        quote! {},
        "is_variant".into(),
        AttrParams {
            enum_: vec!["ignore"],
            variant: vec!["ignore"],
            struct_: vec!["ignore"],
            field: vec!["ignore"],
        },
    )?;
    assert!(
        state.derive_type == DeriveType::Enum,
        "IsVariant can only be derived for enums",
    );

    let enum_name = &input.ident;
    let (imp_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let mut funcs = vec![];
    for variant_state in state.enabled_variant_data().variant_states {
        let variant = variant_state.variant.unwrap();
        let fn_name = format_ident!(
            "is_{}",
            variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span(),
        );
        let variant_ident = &variant.ident;

        let data_pattern = match variant.fields {
            Fields::Named(_) => quote! { {..} },
            Fields::Unnamed(_) => quote! { (..) },
            Fields::Unit => quote! {},
        };
        let variant_name = stringify!(variant_ident);
        let func = quote! {
            #[doc = "Returns `true` if this value is of type `"]
            #[doc = #variant_name]
            #[doc = "`. Returns `false` otherwise"]
            pub const fn #fn_name(&self) -> bool {
                match self {
                    #enum_name ::#variant_ident #data_pattern => true,
                    _ => false
                }
            }
        };
        funcs.push(func);
    }

    let imp = quote! {
        #[automatically_derived]
        impl #imp_generics #enum_name #type_generics #where_clause {
            #(#funcs)*
        }
    };

    Ok(imp)
}
