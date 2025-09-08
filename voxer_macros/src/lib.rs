use crate::common::AttributeArgs;
use syn::parse::Parser;
use syn::parse_macro_input;
use syn::token::Token;

mod common;
mod network;

#[proc_macro_attribute]
pub fn network_message(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input_struct = parse_macro_input!(input as syn::DeriveInput);
    let args = parse_macro_input!(args as AttributeArgs);

    let config =
        network::parse_network_message_config(args).expect("Failed to parse network message args");
    
    let name = &input_struct.ident;

    let fragment_count = match config.fragment_count {
        network::FragmentCount::Auto => quote::quote! { 1 },
        network::FragmentCount::Fixed(expr) => quote::quote! { #expr },
    };
    let tag = config.tag;

    let impl_tokens = quote::quote! {
        #input_struct

        impl crate::voxer_network::NetworkMessageConfig for #name {
            fn tag(&self) -> crate::voxer_network::NetworkMessageTag {
                #tag
            }
            fn fragment_count(&self) -> usize {
                #fragment_count
            }
        }
    };

    impl_tokens.into()
}
