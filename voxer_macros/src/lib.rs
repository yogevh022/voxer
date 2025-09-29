use crate::common::AttributeArgs;
use syn::parse::Parser;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::token::Token;
use crate::shader::rust_to_wgsl_code;

mod common;
mod network;
mod shader;

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

    let expanded = quote::quote! {
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

    expanded.into()
}

#[proc_macro_derive(ShaderType)]
pub fn derive_shader_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_struct = parse_macro_input!(input as syn::DeriveInput);
    let name = &input_struct.ident;

    let wgsl_variant_string = rust_to_wgsl_code(&input_struct);
    let wgsl_variant_literal = syn::LitStr::new(&wgsl_variant_string, input_struct.span());

    let expanded = quote::quote! {
        impl crate::renderer::resources::shader::ShaderType for #name {
            const SHADER_SOURCE: &'static str = #wgsl_variant_literal;
            const SHADER_TYPE_DATA: crate::renderer::resources::shader::VxShaderTypeData = crate::renderer::resources::shader::VxShaderTypeData {
                name: stringify!(#name),
                stride: std::mem::size_of::<#name>()
            };
        }
    };

    expanded.into()
}